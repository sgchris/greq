<?php
/**
 * Configuration Template for Greq Newsletter System
 * 
 * Copy this file to config.php and update the values below
 */

// Admin Authentication
define('ADMIN_USERNAME', 'admin');
define('ADMIN_PASSWORD', 'CHANGE_THIS_PASSWORD'); // Change to a strong password

// CORS Configuration
define('ALLOWED_ORIGINS', [
    'https://greq.me',
    'https://www.greq.me',
    'http://localhost:5173', // Development React
    'http://localhost:3000'  // Alternative development port
]);

// Database Configuration
define('DB_PATH', __DIR__ . '/../database/subscribers.db');

// Security Settings
define('REQUIRE_HTTPS', false); // Set to true in production
define('SESSION_TIMEOUT', 3600); // 1 hour

// Rate Limiting (requests per IP per hour)
define('RATE_LIMIT_ENABLED', false); // Enable in production
define('RATE_LIMIT_MAX_REQUESTS', 10);

?>