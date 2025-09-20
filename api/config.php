<?php
/**
 * Greq API Configuration Script
 * Run this to configure the API settings
 */

require __DIR__ . '/vendor/autoload.php';

class ConfigurationManager
{
    private string $configPath;
    private array $settings;

    public function __construct()
    {
        $this->configPath = __DIR__ . '/config/settings.php';
        $this->settings = require $this->configPath;
    }

    public function interactive(): void
    {
        echo "🛠️  Greq API Configuration\n";
        echo "===========================\n\n";

        echo "Current settings:\n";
        $this->displayCurrentSettings();

        echo "\nWould you like to modify settings? (y/n): ";
        $handle = fopen("php://stdin", "r");
        $response = trim(fgets($handle));
        
        if (strtolower($response) === 'y' || strtolower($response) === 'yes') {
            $this->updateSettings();
        }
        
        fclose($handle);
        
        echo "\n✅ Configuration completed!\n";
    }

    private function displayCurrentSettings(): void
    {
        echo "• Admin Password: " . str_repeat('*', strlen($this->settings['security']['admin_password'])) . "\n";
        echo "• Database: {$this->settings['database']['path']}\n";
        echo "• Log File: {$this->settings['logger']['path']}\n";
        echo "• Rate Limit: {$this->settings['security']['rate_limit']['max_requests']} requests per {$this->settings['security']['rate_limit']['window_minutes']} minutes\n";
        echo "• CORS Origins: " . implode(', ', $this->settings['cors']['allowed_origins']) . "\n";
    }

    private function updateSettings(): void
    {
        echo "\n📝 Update Settings\n";
        echo "==================\n\n";

        // Update admin password
        echo "Enter new admin password (current: " . str_repeat('*', strlen($this->settings['security']['admin_password'])) . "): ";
        $handle = fopen("php://stdin", "r");
        $newPassword = trim(fgets($handle));
        if (!empty($newPassword)) {
            $this->settings['security']['admin_password'] = $newPassword;
            echo "✅ Admin password updated\n";
        }

        // Update rate limit
        echo "\nEnter max requests per window (current: {$this->settings['security']['rate_limit']['max_requests']}): ";
        $maxRequests = trim(fgets($handle));
        if (is_numeric($maxRequests) && $maxRequests > 0) {
            $this->settings['security']['rate_limit']['max_requests'] = (int)$maxRequests;
            echo "✅ Rate limit updated\n";
        }

        // Update window minutes
        echo "Enter rate limit window in minutes (current: {$this->settings['security']['rate_limit']['window_minutes']}): ";
        $windowMinutes = trim(fgets($handle));
        if (is_numeric($windowMinutes) && $windowMinutes > 0) {
            $this->settings['security']['rate_limit']['window_minutes'] = (int)$windowMinutes;
            echo "✅ Rate limit window updated\n";
        }

        fclose($handle);

        // Save settings
        $this->saveSettings();
        echo "\n💾 Settings saved successfully!\n";
    }

    private function saveSettings(): void
    {
        $content = "<?php\n\nreturn " . var_export($this->settings, true) . ";\n";
        file_put_contents($this->configPath, $content);
    }

    public function initializeDatabase(): void
    {
        echo "\n🗄️  Initializing Database\n";
        echo "========================\n";

        try {
            $dbManager = new \Greq\Api\Database\DatabaseManager(
                $this->settings['database'],
                new \Monolog\Logger('config')
            );
            
            $dbManager->initDatabase();
            echo "✅ Database initialized successfully!\n";
            
            // Show current stats
            $subscriberCount = $dbManager->getSubscriberCount();
            echo "📊 Current subscriber count: {$subscriberCount}\n";
            
        } catch (\Exception $e) {
            echo "❌ Database initialization failed: " . $e->getMessage() . "\n";
        }
    }

    public function showInfo(): void
    {
        echo "📋 Greq API Information\n";
        echo "======================\n\n";

        echo "🌐 Server URLs:\n";
        echo "   • Main: http://localhost:8080/\n";
        echo "   • Health: http://localhost:8080/api/health\n";
        echo "   • Subscribe: POST http://localhost:8080/api/subscribe\n";
        echo "   • Subscribers: http://localhost:8080/api/subscribers\n\n";

        echo "🔐 Authentication:\n";
        echo "   • Username: admin\n";
        echo "   • Password: {$this->settings['security']['admin_password']}\n\n";

        echo "📁 Files:\n";
        echo "   • Database: {$this->settings['database']['path']}\n";
        echo "   • Logs: {$this->settings['logger']['path']}\n\n";

        echo "🚀 To start server: php -S localhost:8080 -t public/\n";
        echo "🧪 To run tests: php test.php\n";
    }
}

// CLI interface
if (php_sapi_name() === 'cli') {
    $config = new ConfigurationManager();
    
    $command = $argv[1] ?? 'interactive';
    
    switch ($command) {
        case 'init':
            $config->initializeDatabase();
            break;
        case 'info':
            $config->showInfo();
            break;
        case 'interactive':
        default:
            $config->interactive();
            $config->initializeDatabase();
            $config->showInfo();
            break;
    }
} else {
    echo "This script must be run from the command line.\n";
}