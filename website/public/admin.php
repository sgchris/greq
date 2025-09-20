<?php
/**
 * Simple Database Viewer for Newsletter Subscribers
 * 
 * Basic admin interface to view and manage newsletter subscriptions.
 * Protected with HTTP Basic Authentication.
 */

// Enable error reporting for development
error_reporting(E_ALL);
ini_set('display_errors', 1);

// Load configuration and security
require_once 'config.php';
require_once 'security.php';

// Initialize security with session management
initSecurity(true);

// HTTP Basic Authentication
if (!isset($_SERVER['PHP_AUTH_USER']) || 
    $_SERVER['PHP_AUTH_USER'] !== ADMIN_USERNAME || 
    $_SERVER['PHP_AUTH_PW'] !== ADMIN_PASSWORD) {
    
    logSecurityEvent('admin_auth_failed', ['username' => $_SERVER['PHP_AUTH_USER'] ?? 'none']);
    header('WWW-Authenticate: Basic realm="Greq Newsletter Admin"');
    header('HTTP/1.0 401 Unauthorized');
    echo '<!DOCTYPE html>
<html>
<head>
    <title>Unauthorized</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
        .error { color: #d32f2f; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>🔒 Access Denied</h1>
    <p class="error">Valid credentials required to access the admin panel.</p>
    <p>Please contact the administrator if you need access.</p>
</body>
</html>';
    exit();
}

// Log successful authentication
logSecurityEvent('admin_access_granted', ['username' => $_SERVER['PHP_AUTH_USER']]);

$dbPath = DB_PATH;

try {
    $pdo = new PDO('sqlite:' . $dbPath);
    $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    
    // Get all subscribers
    $stmt = $pdo->query("SELECT * FROM subscribers ORDER BY subscribed_at DESC");
    $subscribers = $stmt->fetchAll(PDO::FETCH_ASSOC);
    
    // Get subscriber count
    $countStmt = $pdo->query("SELECT COUNT(*) as count FROM subscribers WHERE status = 'active'");
    $count = $countStmt->fetch(PDO::FETCH_ASSOC)['count'];
    
} catch (PDOException $e) {
    die('Database error: ' . $e->getMessage());
}
?>

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Newsletter Subscribers - Greq</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }
        
        .header {
            background: #4338ca;
            color: white;
            padding: 20px;
        }
        
        .header h1 {
            margin: 0;
            font-size: 24px;
        }
        
        .stats {
            padding: 20px;
            border-bottom: 1px solid #eee;
            background: #f8f9fa;
        }
        
        .stat-item {
            display: inline-block;
            margin-right: 30px;
        }
        
        .stat-number {
            font-size: 32px;
            font-weight: bold;
            color: #4338ca;
        }
        
        .stat-label {
            font-size: 14px;
            color: #666;
        }
        
        .table-container {
            overflow-x: auto;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
        }
        
        th, td {
            text-align: left;
            padding: 12px 20px;
            border-bottom: 1px solid #eee;
        }
        
        th {
            background-color: #f8f9fa;
            font-weight: 600;
            color: #333;
        }
        
        .status-active {
            color: #059669;
            font-weight: 500;
        }
        
        .email {
            font-family: monospace;
            font-size: 14px;
        }
        
        .date {
            color: #666;
            font-size: 14px;
        }
        
        .empty-state {
            text-align: center;
            padding: 40px;
            color: #666;
        }
        
        .refresh-btn {
            background: #4338ca;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
            margin: 20px;
        }
        
        .refresh-btn:hover {
            background: #3730a3;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📧 Newsletter Subscribers</h1>
            <p>Greq API Testing Tool - Subscriber Management</p>
        </div>
        
        <div class="stats">
            <div class="stat-item">
                <div class="stat-number"><?= $count ?></div>
                <div class="stat-label">Active Subscribers</div>
            </div>
            <div class="stat-item">
                <div class="stat-number"><?= count($subscribers) ?></div>
                <div class="stat-label">Total Records</div>
            </div>
        </div>
        
        <button class="refresh-btn" onclick="location.reload()">🔄 Refresh</button>
        
        <div class="table-container">
            <?php if (empty($subscribers)): ?>
                <div class="empty-state">
                    <h3>No subscribers yet</h3>
                    <p>Newsletter subscriptions will appear here when users sign up.</p>
                </div>
            <?php else: ?>
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Email Address</th>
                            <th>Subscribed At</th>
                            <th>IP Address</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($subscribers as $subscriber): ?>
                            <tr>
                                <td><?= htmlspecialchars($subscriber['id']) ?></td>
                                <td class="email"><?= htmlspecialchars($subscriber['email']) ?></td>
                                <td class="date"><?= date('Y-m-d H:i:s', strtotime($subscriber['subscribed_at'])) ?></td>
                                <td><?= htmlspecialchars($subscriber['ip_address']) ?></td>
                                <td class="status-<?= $subscriber['status'] ?>"><?= ucfirst($subscriber['status']) ?></td>
                            </tr>
                        <?php endforeach; ?>
                    </tbody>
                </table>
            <?php endif; ?>
        </div>
    </div>
    
    <script>
        // Auto-refresh every 30 seconds
        setTimeout(() => {
            location.reload();
        }, 30000);
    </script>
</body>
</html>