<?php
/**
 * Security Helper Functions for Greq Newsletter System
 * 
 * Implements HTTPS enforcement, session management, and rate limiting.
 */

// Load configuration
require_once 'config.php';

/**
 * Enforce HTTPS connections in production
 */
function enforceHTTPS() {
    // Only enforce HTTPS when running in web server context
    if (REQUIRE_HTTPS && !isHTTPS() && isset($_SERVER['HTTP_HOST'])) {
        $redirectURL = 'https://' . $_SERVER['HTTP_HOST'] . $_SERVER['REQUEST_URI'];
        header("Location: $redirectURL", true, 301);
        exit('Redirecting to secure connection...');
    }
}

/**
 * Check if the current connection is HTTPS
 */
function isHTTPS() {
    return (
        (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') ||
        (!empty($_SERVER['SERVER_PORT']) && $_SERVER['SERVER_PORT'] == 443) ||
        (!empty($_SERVER['HTTP_X_FORWARDED_PROTO']) && $_SERVER['HTTP_X_FORWARDED_PROTO'] === 'https') ||
        (!empty($_SERVER['HTTP_X_FORWARDED_SSL']) && $_SERVER['HTTP_X_FORWARDED_SSL'] === 'on')
    );
}

/**
 * Initialize secure session with timeout
 */
function initSecureSession() {
    // Configure session security
    ini_set('session.cookie_secure', REQUIRE_HTTPS ? '1' : '0');
    ini_set('session.cookie_httponly', '1');
    ini_set('session.use_strict_mode', '1');
    ini_set('session.cookie_samesite', 'Strict');
    
    // Set session timeout
    ini_set('session.gc_maxlifetime', SESSION_TIMEOUT);
    
    if (session_status() === PHP_SESSION_NONE) {
        session_start();
    }
    
    // Check for session timeout
    if (isset($_SESSION['last_activity'])) {
        if ((time() - $_SESSION['last_activity']) > SESSION_TIMEOUT) {
            session_unset();
            session_destroy();
            session_start();
            return false; // Session expired
        }
    }
    
    $_SESSION['last_activity'] = time();
    return true;
}

/**
 * Rate limiting implementation
 * Returns true if request is allowed, false if rate limit exceeded
 */
function checkRateLimit($identifier = null) {
    if (!RATE_LIMIT_ENABLED) {
        return true;
    }
    
    // Use IP address if no identifier provided
    if ($identifier === null) {
        $identifier = getClientIP();
    }
    
    $rateLimitFile = __DIR__ . '/../temp/rate_limits.json';
    $currentTime = time();
    $timeWindow = 3600; // 1 hour window
    
    // Create temp directory if it doesn't exist
    $tempDir = dirname($rateLimitFile);
    if (!is_dir($tempDir)) {
        mkdir($tempDir, 0755, true);
    }
    
    // Load existing rate limit data
    $rateLimits = [];
    if (file_exists($rateLimitFile)) {
        $data = file_get_contents($rateLimitFile);
        if ($data) {
            $rateLimits = json_decode($data, true) ?: [];
        }
    }
    
    // Clean up old entries (older than time window)
    $rateLimits = array_filter($rateLimits, function($entry) use ($currentTime, $timeWindow) {
        return ($currentTime - $entry['first_request']) < $timeWindow;
    });
    
    // Check current IP's rate limit
    if (!isset($rateLimits[$identifier])) {
        $rateLimits[$identifier] = [
            'count' => 1,
            'first_request' => $currentTime,
            'last_request' => $currentTime
        ];
    } else {
        // Check if we're still within the time window
        if (($currentTime - $rateLimits[$identifier]['first_request']) >= $timeWindow) {
            // Reset counter for new time window
            $rateLimits[$identifier] = [
                'count' => 1,
                'first_request' => $currentTime,
                'last_request' => $currentTime
            ];
        } else {
            // Increment counter
            $rateLimits[$identifier]['count']++;
            $rateLimits[$identifier]['last_request'] = $currentTime;
            
            // Check if rate limit exceeded
            if ($rateLimits[$identifier]['count'] > RATE_LIMIT_MAX_REQUESTS) {
                // Save updated rate limits
                file_put_contents($rateLimitFile, json_encode($rateLimits), LOCK_EX);
                return false;
            }
        }
    }
    
    // Save updated rate limits
    file_put_contents($rateLimitFile, json_encode($rateLimits), LOCK_EX);
    return true;
}

/**
 * Get client IP address (handles proxies and load balancers)
 */
function getClientIP() {
    $ipKeys = ['HTTP_CF_CONNECTING_IP', 'HTTP_X_FORWARDED_FOR', 'HTTP_X_FORWARDED', 
               'HTTP_X_CLUSTER_CLIENT_IP', 'HTTP_FORWARDED_FOR', 'HTTP_FORWARDED', 'REMOTE_ADDR'];
    
    foreach ($ipKeys as $key) {
        if (!empty($_SERVER[$key])) {
            $ips = explode(',', $_SERVER[$key]);
            $ip = trim($ips[0]);
            
            if (filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE)) {
                return $ip;
            }
        }
    }
    
    return $_SERVER['REMOTE_ADDR'] ?? 'unknown';
}

/**
 * Send rate limit exceeded response
 */
function sendRateLimitResponse() {
    http_response_code(429);
    header('Retry-After: 3600'); // 1 hour
    header('Content-Type: application/json');
    
    $response = [
        'error' => 'Rate limit exceeded',
        'message' => 'Too many requests. Please try again in 1 hour.',
        'retry_after' => 3600
    ];
    
    echo json_encode($response);
    exit();
}

/**
 * Log security events
 */
function logSecurityEvent($event, $details = []) {
    $logFile = __DIR__ . '/../logs/security.log';
    $logDir = dirname($logFile);
    
    // Create logs directory if it doesn't exist
    if (!is_dir($logDir)) {
        mkdir($logDir, 0755, true);
    }
    
    $logEntry = [
        'timestamp' => date('c'),
        'event' => $event,
        'ip' => getClientIP(),
        'user_agent' => $_SERVER['HTTP_USER_AGENT'] ?? 'unknown',
        'request_uri' => $_SERVER['REQUEST_URI'] ?? 'unknown',
        'details' => $details
    ];
    
    $logLine = json_encode($logEntry) . "\n";
    file_put_contents($logFile, $logLine, FILE_APPEND | LOCK_EX);
}

/**
 * Validate and sanitize input to prevent common attacks
 */
function sanitizeInput($input, $type = 'string') {
    if (is_array($input)) {
        return array_map(function($item) use ($type) {
            return sanitizeInput($item, $type);
        }, $input);
    }
    
    switch ($type) {
        case 'email':
            return filter_var(trim($input), FILTER_SANITIZE_EMAIL);
        case 'int':
            return filter_var($input, FILTER_SANITIZE_NUMBER_INT);
        case 'float':
            return filter_var($input, FILTER_SANITIZE_NUMBER_FLOAT, FILTER_FLAG_ALLOW_FRACTION);
        case 'url':
            return filter_var(trim($input), FILTER_SANITIZE_URL);
        case 'string':
        default:
            return htmlspecialchars(trim($input), ENT_QUOTES, 'UTF-8');
    }
}

/**
 * Generate secure headers
 */
function setSecurityHeaders() {
    // Prevent XSS
    header('X-Content-Type-Options: nosniff');
    header('X-Frame-Options: DENY');
    header('X-XSS-Protection: 1; mode=block');
    header('Referrer-Policy: strict-origin-when-cross-origin');
    
    // Content Security Policy
    $csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'";
    header("Content-Security-Policy: $csp");
    
    // HSTS (only for HTTPS)
    if (isHTTPS()) {
        header('Strict-Transport-Security: max-age=31536000; includeSubDomains; preload');
    }
}

/**
 * Complete security initialization
 * Call this at the start of each protected script
 */
function initSecurity($requireAuth = false) {
    // Enforce HTTPS
    enforceHTTPS();
    
    // Set security headers
    setSecurityHeaders();
    
    // Check rate limiting
    if (!checkRateLimit()) {
        logSecurityEvent('rate_limit_exceeded');
        sendRateLimitResponse();
    }
    
    // Initialize secure session if authentication required
    if ($requireAuth) {
        if (!initSecureSession()) {
            logSecurityEvent('session_expired');
        }
    }
}

?>