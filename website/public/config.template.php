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
define('REQUIRE_HTTPS', true); // Set to true in production - enforces HTTPS redirects
define('SESSION_TIMEOUT', 3600); // Session timeout in seconds (1 hour default)

// Rate Limiting Configuration
define('RATE_LIMIT_ENABLED', true); // Enable rate limiting in production
define('RATE_LIMIT_MAX_REQUESTS', 5); // Max requests per IP per hour

// Security Features Implemented:
// - HTTPS enforcement with automatic redirects
// - Secure session management with timeout
// - Rate limiting per IP address (configurable window)
// - Security headers (CSP, HSTS, XSS protection)
// - Input sanitization and validation
// - Security event logging
// - Protection against common web attacks

?>