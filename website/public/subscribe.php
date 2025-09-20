<?php
/**
 * Newsletter Subscription Handler
 * 
 * Handles newsletter subscription requests and stores them in SQLite database.
 * No frameworks, pure PHP implementation.
 */

// Load configuration and security
require_once 'config.php';
require_once 'security.php';

// Initialize security (HTTPS, rate limiting, headers)
initSecurity();

// Set CORS headers for production and development
$origin = $_SERVER['HTTP_ORIGIN'] ?? '';

// Check if origin is greq.me or any subdomain, or in allowed origins list
if (preg_match('/^https?:\/\/([a-zA-Z0-9\-]+\.)?greq\.me$/', $origin) || in_array($origin, ALLOWED_ORIGINS)) {
    header('Access-Control-Allow-Origin: ' . $origin);
}

header('Access-Control-Allow-Methods: POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type, Accept');
header('Access-Control-Allow-Credentials: true');
header('Content-Type: application/json');

// Handle preflight requests
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(200);
    exit();
}

// Only allow POST requests
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['error' => 'Method not allowed']);
    exit();
}

/**
 * Initialize SQLite database and create table if it doesn't exist
 */
function initDatabase() {
    $dbPath = DB_PATH;
    
    try {
        $pdo = new PDO('sqlite:' . $dbPath);
        $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
        
        // Create subscribers table if it doesn't exist
        $sql = "CREATE TABLE IF NOT EXISTS subscribers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            ip_address TEXT,
            user_agent TEXT,
            status TEXT DEFAULT 'active'
        )";
        
        $pdo->exec($sql);
        
        return $pdo;
    } catch (PDOException $e) {
        error_log('Database error: ' . $e->getMessage());
        return null;
    }
}

/**
 * Validate email address
 */
function validateEmail($email) {
    if (empty($email)) {
        return 'Email is required';
    }
    
    if (!filter_var($email, FILTER_VALIDATE_EMAIL)) {
        return 'Invalid email format';
    }
    
    if (strlen($email) > 255) {
        return 'Email address is too long';
    }
    
    return null;
}

/**
 * Check if email already exists in database
 */
function emailExists($pdo, $email) {
    try {
        $stmt = $pdo->prepare("SELECT COUNT(*) FROM subscribers WHERE email = ?");
        $stmt->execute([$email]);
        return $stmt->fetchColumn() > 0;
    } catch (PDOException $e) {
        error_log('Database query error: ' . $e->getMessage());
        return false;
    }
}

/**
 * Add new subscriber to database
 */
function addSubscriber($pdo, $email, $ipAddress, $userAgent) {
    try {
        $stmt = $pdo->prepare("
            INSERT INTO subscribers (email, ip_address, user_agent) 
            VALUES (?, ?, ?)
        ");
        
        return $stmt->execute([$email, $ipAddress, $userAgent]);
    } catch (PDOException $e) {
        error_log('Database insert error: ' . $e->getMessage());
        return false;
    }
}

// Main execution starts here
try {
    // Get JSON input
    $input = json_decode(file_get_contents('php://input'), true);
    
    if (json_last_error() !== JSON_ERROR_NONE) {
        http_response_code(400);
        echo json_encode(['error' => 'Invalid JSON']);
        exit();
    }
    
    $email = sanitizeInput($input['email'] ?? '', 'email');
    
    // Validate email
    $emailError = validateEmail($email);
    if ($emailError) {
        http_response_code(400);
        echo json_encode(['error' => $emailError]);
        exit();
    }
    
    // Initialize database
    $pdo = initDatabase();
    if (!$pdo) {
        http_response_code(500);
        echo json_encode(['error' => 'Database connection failed']);
        exit();
    }
    
    // Check if email already exists
    if (emailExists($pdo, $email)) {
        http_response_code(409);
        echo json_encode(['error' => 'Email already subscribed']);
        exit();
    }
    
    // Get client information
    $ipAddress = $_SERVER['REMOTE_ADDR'] ?? 'unknown';
    $userAgent = $_SERVER['HTTP_USER_AGENT'] ?? 'unknown';
    
    // Add subscriber
    if (addSubscriber($pdo, $email, $ipAddress, $userAgent)) {
        logSecurityEvent('newsletter_subscription', ['email' => $email]);
        http_response_code(201);
        echo json_encode([
            'success' => true,
            'message' => 'Successfully subscribed to newsletter'
        ]);
    } else {
        http_response_code(500);
        echo json_encode(['error' => 'Failed to subscribe']);
    }
    
} catch (Exception $e) {
    error_log('Subscription error: ' . $e->getMessage());
    http_response_code(500);
    echo json_encode(['error' => 'Internal server error']);
}
?>