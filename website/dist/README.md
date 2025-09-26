# Newsletter Subscription API

A simple PHP-based newsletter subscription system that stores email addresses in an SQLite database.

## Setup Instructions

### Prerequisites
- PHP 7.4 or higher with SQLite extension enabled
- The React development server running on `http://localhost:5173`

### Starting the PHP Server

1. **Navigate to the public directory**:
   ```bash
   cd website/public
   ```

2. **Start the PHP built-in server**:
   ```bash
   php -S localhost:8080
   ```
   
   The server will start on `http://localhost:8080`

### API Endpoints

#### POST /subscribe.php
Subscribes an email address to the newsletter.

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**Response (Success - 201):**
```json
{
  "success": true,
  "message": "Successfully subscribed to newsletter"
}
```

**Response (Error - 400):**
```json
{
  "error": "Invalid email format"
}
```

**Response (Already Exists - 409):**
```json
{
  "error": "Email already subscribed"
}
```

### Database Structure

The SQLite database is automatically created at `website/database/subscribers.db` with the following table:

```sql
CREATE TABLE subscribers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE NOT NULL,
    subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT,
    status TEXT DEFAULT 'active'
);
```

### CORS Configuration

The API is configured to allow requests from:
- `http://localhost:5173` (React development server)

For production, update the CORS headers in `subscribe.php` to match your domain.

### Testing the API

You can test the API using curl:

```bash
# Subscribe a new email
curl -X POST http://localhost:8080/subscribe.php \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com"}'

# Try subscribing the same email again (should return 409)
curl -X POST http://localhost:8080/subscribe.php \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com"}'
```

### Database Management

To view subscribers, you can use any SQLite client or the command line:

```bash
cd website/database
sqlite3 subscribers.db "SELECT * FROM subscribers;"
```

### Security Considerations

This is a development implementation. For production use, consider:

1. **Input Sanitization**: Additional validation and sanitization
2. **Rate Limiting**: Prevent spam subscriptions
3. **Email Verification**: Send confirmation emails
4. **HTTPS**: Use SSL/TLS in production
5. **Error Logging**: Configure proper error logging
6. **Database Security**: Proper file permissions and backup strategy

### Troubleshooting

**Database Permission Issues:**
```bash
# Ensure the database directory is writable
chmod 755 website/database
```

**CORS Issues:**
- Make sure the React app is running on `http://localhost:5173`
- Update CORS headers if using a different port

**PHP Extensions:**
```bash
# Check if SQLite is enabled
php -m | grep sqlite
```