# Greq Newsletter System - Security Implementation

## Overview

The Greq Newsletter System now includes comprehensive security mechanisms as defined in the configuration file. This document outlines all implemented security features and their configuration.

## Implemented Security Features

### 1. HTTPS Enforcement (`REQUIRE_HTTPS`)

**Purpose**: Forces all connections to use HTTPS in production environments.

**Configuration**: 
```php
define('REQUIRE_HTTPS', true); // Set to true in production
```

**Implementation**:
- Automatically redirects HTTP requests to HTTPS with 301 status
- Detects HTTPS through multiple methods (server port, headers, proxy headers)
- Only enforced when `REQUIRE_HTTPS` is set to `true`
- Handles various server configurations and reverse proxies

**Files**: `security.php` (enforceHTTPS function)

### 2. Session Management with Timeout (`SESSION_TIMEOUT`)

**Purpose**: Secure session handling with automatic timeout for admin access.

**Configuration**:
```php
define('SESSION_TIMEOUT', 3600); // 1 hour in seconds
```

**Implementation**:
- Secure session configuration (httponly, secure, strict SameSite)
- Automatic session expiration based on inactivity
- Session regeneration on timeout
- Secure cookie settings for production

**Files**: `security.php` (initSecureSession function), `admin.php`

### 3. Rate Limiting (`RATE_LIMIT_ENABLED`, `RATE_LIMIT_MAX_REQUESTS`)

**Purpose**: Prevents abuse and DoS attacks by limiting requests per IP.

**Configuration**:
```php
define('RATE_LIMIT_ENABLED', true); // Enable in production
define('RATE_LIMIT_MAX_REQUESTS', 5); // Max requests per hour per IP
```

**Implementation**:
- Per-IP address rate limiting with 1-hour time window
- Persistent storage in JSON file (`temp/rate_limits.json`)
- Automatic cleanup of expired rate limit entries
- Returns HTTP 429 (Too Many Requests) when exceeded
- Handles proxy and load balancer IP detection

**Files**: `security.php` (checkRateLimit function)

### 4. Security Headers

**Purpose**: Protection against common web attacks (XSS, clickjacking, etc.).

**Implementation**:
- `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
- `X-Frame-Options: DENY` - Prevents clickjacking
- `X-XSS-Protection: 1; mode=block` - Browser XSS protection
- `Content-Security-Policy` - Restricts resource loading
- `Strict-Transport-Security` - HSTS header for HTTPS connections
- `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer info

**Files**: `security.php` (setSecurityHeaders function)

### 5. Input Sanitization and Validation

**Purpose**: Prevents injection attacks and ensures data integrity.

**Implementation**:
- Email sanitization and validation
- HTML entity encoding for output
- Type-specific sanitization (string, email, int, float, URL)
- Protection against SQL injection (parameterized queries)

**Files**: `security.php` (sanitizeInput function), `subscribe.php`

### 6. Security Event Logging

**Purpose**: Audit trail for security events and attempted attacks.

**Implementation**:
- Comprehensive logging of security events
- Structured JSON log format
- IP address and user agent tracking
- Timestamp and request details
- Log file: `logs/security.log`

**Events Logged**:
- Rate limit exceeded
- Admin authentication failures
- Admin access granted
- Newsletter subscriptions
- Session timeouts

**Files**: `security.php` (logSecurityEvent function)

## File Structure

```
website/
├── public/
│   ├── security.php          # Main security implementation
│   ├── config.php            # Security configuration (protected)
│   ├── config.template.php   # Configuration template
│   ├── subscribe.php         # Newsletter API with security
│   ├── admin.php            # Admin panel with authentication
│   └── index.php            # API info with security headers
├── temp/
│   └── rate_limits.json     # Rate limiting data (auto-generated)
├── logs/
│   └── security.log         # Security event log (auto-generated)
└── .gitignore              # Protects sensitive files
```

## Usage Instructions

### 1. Initialize Security

Call at the start of each protected script:

```php
require_once 'security.php';

// Basic security (HTTPS, rate limiting, headers)
initSecurity();

// With session management (for admin pages)
initSecurity(true);
```

### 2. Rate Limiting

Automatic for all requests. Custom identifiers can be used:

```php
// Check rate limit for specific identifier
if (!checkRateLimit('custom_identifier')) {
    sendRateLimitResponse();
}
```

### 3. Input Sanitization

```php
// Sanitize different types of input
$email = sanitizeInput($userInput, 'email');
$text = sanitizeInput($userInput, 'string');
$number = sanitizeInput($userInput, 'int');
```

### 4. Security Logging

```php
// Log security events
logSecurityEvent('event_type', ['detail1' => 'value1']);
```

## Configuration for Production

### Required Settings

1. **Enable HTTPS enforcement**:
   ```php
   define('REQUIRE_HTTPS', true);
   ```

2. **Enable rate limiting**:
   ```php
   define('RATE_LIMIT_ENABLED', true);
   define('RATE_LIMIT_MAX_REQUESTS', 5); // Adjust as needed
   ```

3. **Set appropriate session timeout**:
   ```php
   define('SESSION_TIMEOUT', 3600); // 1 hour
   ```

4. **Change default admin credentials**:
   ```php
   define('ADMIN_USERNAME', 'your_admin_username');
   define('ADMIN_PASSWORD', 'your_secure_password');
   ```

### File Permissions

Ensure proper file permissions:
- `temp/` directory: writable (755)
- `logs/` directory: writable (755)
- `config.php`: protected (644)

### Monitoring

Monitor the security log file for:
- Repeated authentication failures
- Rate limit violations
- Unusual access patterns

## Security Best Practices

1. **Regular Updates**: Keep PHP and server software updated
2. **Strong Passwords**: Use strong, unique passwords for admin access
3. **HTTPS Certificate**: Use valid SSL/TLS certificates in production
4. **Log Monitoring**: Regularly review security logs
5. **Backup**: Regular backups of configuration and database
6. **Access Control**: Restrict server access to necessary personnel

## Testing Security Features

### 1. Test HTTPS Enforcement

Visit `http://localhost:8080` (should redirect to HTTPS in production)

### 2. Test Rate Limiting

Make more than 5 requests per hour to trigger rate limiting

### 3. Test Authentication

Access `/admin.php` without credentials (should prompt for authentication)

### 4. Test Security Headers

Check response headers with browser developer tools

### 5. Monitor Logs

Check `logs/security.log` for recorded events

## Troubleshooting

### Common Issues

1. **Rate Limiting False Positives**: 
   - Check IP detection for proxy configurations
   - Adjust `RATE_LIMIT_MAX_REQUESTS` if needed

2. **Session Timeout Too Aggressive**:
   - Increase `SESSION_TIMEOUT` value

3. **HTTPS Redirect Loop**:
   - Check server configuration and proxy headers
   - Verify `isHTTPS()` detection logic

4. **File Permission Issues**:
   - Ensure `temp/` and `logs/` directories are writable
   - Check file ownership and permissions

### Log Analysis

Security events are logged in JSON format for easy parsing:

```json
{
  "timestamp": "2025-09-20T11:30:00+00:00",
  "event": "rate_limit_exceeded",
  "ip": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "request_uri": "/subscribe.php",
  "details": {}
}
```

This comprehensive security implementation provides enterprise-level protection while maintaining ease of use and configuration flexibility.