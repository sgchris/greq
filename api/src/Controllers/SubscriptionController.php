<?php

namespace Greq\Api\Controllers;

use Greq\Api\Database\DatabaseManager;
use Greq\Api\Middleware\SecurityMiddleware;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Log\LoggerInterface;

/**
 * Controller for subscription-related endpoints
 */
class SubscriptionController
{
    private DatabaseManager $db;
    private LoggerInterface $logger;

    public function __construct(DatabaseManager $db, LoggerInterface $logger)
    {
        $this->db = $db;
        $this->logger = $logger;
    }

    /**
     * Handle subscription requests
     */
    public function subscribe(Request $request, Response $response): Response
    {
        try {
            $body = $request->getBody()->getContents();
            $data = json_decode($body, true);
            
            if (json_last_error() !== JSON_ERROR_NONE) {
                return $this->errorResponse($response, 'Invalid JSON', 400);
            }
            
            $email = $data['email'] ?? '';
            
            // Validate email
            if (empty($email)) {
                return $this->errorResponse($response, 'Email is required', 400);
            }
            
            // Sanitize email
            $email = SecurityMiddleware::sanitizeInput($email);
            
            if (!SecurityMiddleware::validateEmail($email)) {
                return $this->errorResponse($response, 'Invalid email format', 400);
            }
            
            // Check if email already exists
            if ($this->db->emailExists($email)) {
                return $this->errorResponse($response, 'Email already subscribed', 409);
            }
            
            // Get client information
            $ipAddress = $this->getClientIp($request);
            $userAgent = $request->getHeaderLine('User-Agent');
            
            // Add subscriber
            if ($this->db->addSubscriber($email, $ipAddress, $userAgent)) {
                $responseData = [
                    'success' => true,
                    'message' => 'Successfully subscribed to newsletter'
                ];
                
                $response->getBody()->write(json_encode($responseData));
                return $response
                    ->withHeader('Content-Type', 'application/json')
                    ->withStatus(201);
            }
            
            return $this->errorResponse($response, 'Failed to subscribe', 500);
            
        } catch (\Exception $e) {
            $this->logger->error('Subscription error', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);
            
            return $this->errorResponse($response, 'Internal server error', 500);
        }
    }

    /**
     * Get subscribers list (HTML view)
     */
    public function getSubscribers(Request $request, Response $response): Response
    {
        try {
            $subscribers = $this->db->getSubscribers();
            $count = $this->db->getSubscriberCount();
            
            $html = $this->renderSubscribersHtml($subscribers, $count);
            
            $response->getBody()->write($html);
            return $response->withHeader('Content-Type', 'text/html');
            
        } catch (\Exception $e) {
            $this->logger->error('Get subscribers error', [
                'error' => $e->getMessage()
            ]);
            
            return $this->errorResponse($response, 'Failed to retrieve subscribers', 500);
        }
    }

    /**
     * Handle OPTIONS requests for CORS
     */
    public function options(Request $request, Response $response): Response
    {
        return $response->withStatus(200);
    }

    private function errorResponse(Response $response, string $message, int $status = 400): Response
    {
        $errorData = [
            'error' => $message,
            'status' => $status
        ];
        
        $response->getBody()->write(json_encode($errorData));
        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withStatus($status);
    }

    private function getClientIp(Request $request): string
    {
        $serverParams = $request->getServerParams();
        
        $forwardedFor = $request->getHeaderLine('X-Forwarded-For');
        if (!empty($forwardedFor)) {
            $ips = explode(',', $forwardedFor);
            return trim($ips[0]);
        }
        
        return $serverParams['REMOTE_ADDR'] ?? 'unknown';
    }

    private function renderSubscribersHtml(array $subscribers, int $count): string
    {
        $subscriberRows = '';
        
        if (empty($subscribers)) {
            $subscriberRows = '<tr><td colspan="5" class="px-6 py-4 text-center text-gray-500">No subscribers yet</td></tr>';
        } else {
            foreach ($subscribers as $subscriber) {
                $subscriberRows .= sprintf(
                    '<tr class="hover:bg-gray-50">
                        <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">%d</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 font-mono">%s</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">%s</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">%s</td>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800">
                                %s
                            </span>
                        </td>
                    </tr>',
                    htmlspecialchars($subscriber['id']),
                    htmlspecialchars($subscriber['email']),
                    htmlspecialchars(date('Y-m-d H:i:s', strtotime($subscriber['subscribed_at']))),
                    htmlspecialchars($subscriber['ip_address'] ?? 'unknown'),
                    htmlspecialchars(ucfirst($subscriber['status']))
                );
            }
        }

        return <<<HTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Greq Newsletter Subscribers</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/heroicons/1.0.6/outline.min.css">
</head>
<body class="bg-gray-100 min-h-screen">
    <div class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <!-- Header -->
        <div class="bg-white shadow rounded-lg mb-6">
            <div class="px-6 py-4">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">📧 Newsletter Subscribers</h1>
                        <p class="text-gray-600 mt-1">Greq API Service - Subscriber Management</p>
                    </div>
                    <button 
                        onclick="location.reload()" 
                        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                    >
                        🔄 Refresh
                    </button>
                </div>
            </div>
        </div>

        <!-- Stats -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
            <div class="bg-white overflow-hidden shadow rounded-lg">
                <div class="p-5">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                                <span class="text-white text-sm font-bold">👥</span>
                            </div>
                        </div>
                        <div class="ml-5 w-0 flex-1">
                            <dl>
                                <dt class="text-sm font-medium text-gray-500 truncate">Active Subscribers</dt>
                                <dd class="text-lg font-medium text-gray-900">{$count}</dd>
                            </dl>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-white overflow-hidden shadow rounded-lg">
                <div class="p-5">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                                <span class="text-white text-sm font-bold">📊</span>
                            </div>
                        </div>
                        <div class="ml-5 w-0 flex-1">
                            <dl>
                                <dt class="text-sm font-medium text-gray-500 truncate">Total Records</dt>
                                <dd class="text-lg font-medium text-gray-900">" . count($subscribers) . "</dd>
                            </dl>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bg-white overflow-hidden shadow rounded-lg">
                <div class="p-5">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center">
                                <span class="text-white text-sm font-bold">🕒</span>
                            </div>
                        </div>
                        <div class="ml-5 w-0 flex-1">
                            <dl>
                                <dt class="text-sm font-medium text-gray-500 truncate">Last Updated</dt>
                                <dd class="text-lg font-medium text-gray-900">" . date('H:i:s') . "</dd>
                            </dl>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Subscribers Table -->
        <div class="bg-white shadow overflow-hidden sm:rounded-lg">
            <div class="px-6 py-4 border-b border-gray-200">
                <h2 class="text-lg font-medium text-gray-900">Subscriber Details</h2>
            </div>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">ID</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Email</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Subscribed At</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">IP Address</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        {$subscriberRows}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Footer -->
        <div class="mt-6 text-center text-sm text-gray-500">
            <p>Greq API Service v1.0.0 - Auto-refresh every 30 seconds</p>
        </div>
    </div>

    <script>
        // Auto-refresh every 30 seconds
        setTimeout(() => {
            location.reload();
        }, 30000);
        
        // Add some interactivity
        document.addEventListener('DOMContentLoaded', function() {
            const rows = document.querySelectorAll('tbody tr');
            rows.forEach(row => {
                row.addEventListener('click', function() {
                    this.classList.toggle('bg-blue-50');
                });
            });
        });
    </script>
</body>
</html>
HTML;
    }
}