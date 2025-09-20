<?php
/**
 * Basic index file for Greq Newsletter Backend
 * 
 * Provides basic information about the API and handles favicon requests.
 */

// Load configuration and security
require_once 'config.php';
require_once 'security.php';

// Initialize security
initSecurity();

// Handle favicon requests
if (isset($_SERVER['REQUEST_URI']) && $_SERVER['REQUEST_URI'] === '/favicon.ico') {
    $faviconPath = __DIR__ . '/greq_logo.png';
    if (file_exists($faviconPath)) {
        header('Content-Type: image/png');
        header('Cache-Control: public, max-age=86400'); // Cache for 1 day
        readfile($faviconPath);
        exit();
    } else {
        http_response_code(404);
        exit();
    }
}

// Set content type for regular requests
header('Content-Type: application/json');

// API information response
$response = [
    'service' => 'Greq Newsletter API',
    'version' => '1.0.0',
    'status' => 'operational',
    'security' => [
        'https_enforced' => REQUIRE_HTTPS,
        'rate_limiting' => RATE_LIMIT_ENABLED,
        'max_requests_per_hour' => RATE_LIMIT_MAX_REQUESTS
    ],
    'endpoints' => [
        'POST /subscribe.php' => 'Subscribe to newsletter',
        'GET /admin.php' => 'Admin panel (authentication required)'
    ],
    'timestamp' => date('c')
];

echo json_encode($response, JSON_PRETTY_PRINT);
?>