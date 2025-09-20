<?php

use DI\Container;
use DI\ContainerBuilder;
use Greq\Api\Controllers\SubscriptionController;
use Greq\Api\Database\DatabaseManager;
use Greq\Api\Middleware\AuthMiddleware;
use Greq\Api\Middleware\CorsMiddleware;
use Greq\Api\Middleware\RateLimitMiddleware;
use Greq\Api\Middleware\SecurityMiddleware;
use Monolog\Handler\StreamHandler;
use Monolog\Logger;
use Psr\Container\ContainerInterface;
use Psr\Log\LoggerInterface;
use Slim\Factory\AppFactory;
use Slim\Middleware\ErrorMiddleware;

require __DIR__ . '/../vendor/autoload.php';

// Load configuration
$settings = require __DIR__ . '/../config/settings.php';

// Build DI container
$containerBuilder = new ContainerBuilder();

$containerBuilder->addDefinitions([
    'settings' => $settings,
    
    LoggerInterface::class => function (ContainerInterface $c) {
        $settings = $c->get('settings');
        $loggerSettings = $settings['logger'];
        
        $logger = new Logger($loggerSettings['name']);
        
        $processor = new \Monolog\Processor\UidProcessor();
        $logger->pushProcessor($processor);
        
        $handler = new StreamHandler($loggerSettings['path'], $loggerSettings['level']);
        $logger->pushHandler($handler);
        
        return $logger;
    },
    
    DatabaseManager::class => function (ContainerInterface $c) {
        $settings = $c->get('settings');
        $dbSettings = $settings['database'];
        
        return new DatabaseManager($dbSettings, $c->get(LoggerInterface::class));
    },
    
    SubscriptionController::class => function (ContainerInterface $c) {
        return new SubscriptionController(
            $c->get(DatabaseManager::class),
            $c->get(LoggerInterface::class)
        );
    }
]);

$container = $containerBuilder->build();

// Create app
AppFactory::setContainer($container);
$app = AppFactory::create();

// Get settings
$settings = $container->get('settings');

// Add routing middleware
$app->addRoutingMiddleware();

// Add body parsing middleware
$app->addBodyParsingMiddleware();

// Add error middleware
$errorMiddleware = $app->addErrorMiddleware(
    $settings['error']['displayErrorDetails'],
    $settings['error']['logErrors'],
    $settings['error']['logErrorDetails']
);

// Set error handler
$errorMiddleware->setDefaultErrorHandler(function (
    \Psr\Http\Message\ServerRequestInterface $request,
    \Throwable $exception,
    bool $displayErrorDetails,
    bool $logErrors,
    bool $logErrorDetails,
    ?\Psr\Log\LoggerInterface $logger = null
) {
    if ($logger) {
        $logger->error('Application error', [
            'exception' => $exception->getMessage(),
            'file' => $exception->getFile(),
            'line' => $exception->getLine(),
            'trace' => $exception->getTraceAsString()
        ]);
    }
    
    $response = new \Slim\Psr7\Response();
    $errorData = [
        'error' => 'Internal server error',
        'status' => 500
    ];
    
    if ($displayErrorDetails) {
        $errorData['details'] = [
            'message' => $exception->getMessage(),
            'file' => $exception->getFile(),
            'line' => $exception->getLine()
        ];
    }
    
    $response->getBody()->write(json_encode($errorData));
    return $response
        ->withHeader('Content-Type', 'application/json')
        ->withStatus(500);
});

// Add middleware stack (in reverse order of execution)
$app->add(new AuthMiddleware(
    $settings['security']['admin_password'],
    $container->get(LoggerInterface::class)
));
$app->add(new SecurityMiddleware($container->get(LoggerInterface::class)));
$app->add(new RateLimitMiddleware(
    $container->get(DatabaseManager::class),
    $container->get(LoggerInterface::class),
    $settings['security']['rate_limit']
));
$app->add(new CorsMiddleware($settings['cors']));

// Define routes
$app->group('/api', function ($group) {
    // Health check endpoint (no auth required)
    $group->get('/health', function ($request, $response) {
        $data = [
            'status' => 'healthy',
            'timestamp' => date('c'),
            'version' => '1.0.0'
        ];
        
        $response->getBody()->write(json_encode($data));
        return $response->withHeader('Content-Type', 'application/json');
    });
    
    // Newsletter subscription endpoint (no auth required)
    $group->post('/subscribe', SubscriptionController::class . ':subscribe');
    $group->options('/subscribe', SubscriptionController::class . ':options');
    
    // Subscribers management (requires auth)
    $group->get('/subscribers', SubscriptionController::class . ':getSubscribers');
});

// Catch-all route for API
$app->get('/api[/{path:.*}]', function ($request, $response) {
    $data = [
        'error' => 'Endpoint not found',
        'status' => 404,
        'available_endpoints' => [
            'GET /api/health - Health check',
            'POST /api/subscribe - Subscribe to newsletter',
            'GET /api/subscribers - View subscribers (requires auth)'
        ]
    ];
    
    $response->getBody()->write(json_encode($data));
    return $response
        ->withHeader('Content-Type', 'application/json')
        ->withStatus(404);
});

// Root route
$app->get('/', function ($request, $response) {
    $html = <<<HTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Greq API Service</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gradient-to-br from-blue-50 to-indigo-100 min-h-screen">
    <div class="container mx-auto px-4 py-8">
        <div class="max-w-4xl mx-auto">
            <!-- Header -->
            <div class="text-center mb-12">
                <h1 class="text-4xl font-bold text-gray-900 mb-4">🚀 Greq API Service</h1>
                <p class="text-xl text-gray-600">Newsletter Subscription Management API</p>
            </div>

            <!-- Stats Cards -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <div class="bg-white rounded-lg shadow-md p-6 text-center">
                    <div class="text-3xl mb-2">📧</div>
                    <h3 class="text-lg font-semibold text-gray-900">Newsletter API</h3>
                    <p class="text-gray-600">Subscribe & manage users</p>
                </div>
                
                <div class="bg-white rounded-lg shadow-md p-6 text-center">
                    <div class="text-3xl mb-2">🔒</div>
                    <h3 class="text-lg font-semibold text-gray-900">Secure</h3>
                    <p class="text-gray-600">Rate limiting & auth</p>
                </div>
                
                <div class="bg-white rounded-lg shadow-md p-6 text-center">
                    <div class="text-3xl mb-2">⚡</div>
                    <h3 class="text-lg font-semibold text-gray-900">Fast API</h3>
                    <p class="text-gray-600">Slim framework powered</p>
                </div>
            </div>

            <!-- API Documentation -->
            <div class="bg-white rounded-lg shadow-md p-6 mb-8">
                <h2 class="text-2xl font-bold text-gray-900 mb-6">API Endpoints</h2>
                
                <div class="space-y-4">
                    <div class="border border-gray-200 rounded-lg p-4">
                        <div class="flex items-center mb-2">
                            <span class="bg-green-100 text-green-800 text-xs font-medium px-2.5 py-0.5 rounded mr-3">GET</span>
                            <code class="text-sm font-mono text-gray-900">/api/health</code>
                        </div>
                        <p class="text-gray-600 text-sm">Health check endpoint - no authentication required</p>
                    </div>
                    
                    <div class="border border-gray-200 rounded-lg p-4">
                        <div class="flex items-center mb-2">
                            <span class="bg-blue-100 text-blue-800 text-xs font-medium px-2.5 py-0.5 rounded mr-3">POST</span>
                            <code class="text-sm font-mono text-gray-900">/api/subscribe</code>
                        </div>
                        <p class="text-gray-600 text-sm">Subscribe to newsletter - requires JSON body with email field</p>
                    </div>
                    
                    <div class="border border-gray-200 rounded-lg p-4">
                        <div class="flex items-center mb-2">
                            <span class="bg-green-100 text-green-800 text-xs font-medium px-2.5 py-0.5 rounded mr-3">GET</span>
                            <code class="text-sm font-mono text-gray-900">/api/subscribers</code>
                        </div>
                        <p class="text-gray-600 text-sm">View subscribers list - requires HTTP Basic authentication</p>
                    </div>
                </div>
            </div>

            <!-- Quick Actions -->
            <div class="bg-white rounded-lg shadow-md p-6">
                <h2 class="text-2xl font-bold text-gray-900 mb-6">Quick Actions</h2>
                
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <a href="/api/health" 
                       class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-lg text-center transition-colors duration-200 block">
                        🔍 Check API Health
                    </a>
                    
                    <a href="/api/subscribers" 
                       class="bg-green-600 hover:bg-green-700 text-white px-6 py-3 rounded-lg text-center transition-colors duration-200 block">
                        👥 View Subscribers
                    </a>
                </div>
            </div>

            <!-- Footer -->
            <div class="text-center mt-8 text-gray-600">
                <p>Greq API Service v1.0.0 - Built with Slim Framework</p>
            </div>
        </div>
    </div>
</body>
</html>
HTML;
    
    $response->getBody()->write($html);
    return $response->withHeader('Content-Type', 'text/html');
});

// Initialize database
$db = $container->get(DatabaseManager::class);
$db->initDatabase();

return $app;