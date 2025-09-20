<?php

namespace Greq\Api\Middleware;

use Greq\Api\Database\DatabaseManager;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;
use Slim\Psr7\Factory\ResponseFactory;

/**
 * Rate limiting middleware
 */
class RateLimitMiddleware implements MiddlewareInterface
{
    private DatabaseManager $db;
    private LoggerInterface $logger;
    private array $config;
    private ResponseFactory $responseFactory;

    public function __construct(DatabaseManager $db, LoggerInterface $logger, array $config)
    {
        $this->db = $db;
        $this->logger = $logger;
        $this->config = $config;
        $this->responseFactory = new ResponseFactory();
    }

    public function process(Request $request, RequestHandler $handler): Response
    {
        if (!$this->config['enabled']) {
            return $handler->handle($request);
        }

        $ipAddress = $this->getClientIp($request);
        
        // Clean old rate limit entries
        $this->db->cleanOldRateLimits();
        
        // Update and get current request count for this IP
        $requestCount = $this->db->updateRateLimit($ipAddress);
        
        if ($requestCount > $this->config['max_requests']) {
            $this->logger->warning('Rate limit exceeded', [
                'ip' => $ipAddress,
                'count' => $requestCount,
                'limit' => $this->config['max_requests']
            ]);
            
            $response = $this->responseFactory->createResponse(429);
            $response->getBody()->write(json_encode([
                'error' => 'Rate limit exceeded',
                'message' => 'Too many requests. Try again later.',
                'retry_after' => $this->config['time_window']
            ]));
            
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withHeader('Retry-After', (string) $this->config['time_window']);
        }
        
        return $handler->handle($request);
    }

    private function getClientIp(Request $request): string
    {
        $serverParams = $request->getServerParams();
        
        // Check for forwarded IP from proxy
        $forwardedFor = $request->getHeaderLine('X-Forwarded-For');
        if (!empty($forwardedFor)) {
            $ips = explode(',', $forwardedFor);
            return trim($ips[0]);
        }
        
        // Check for real IP header
        $realIp = $request->getHeaderLine('X-Real-IP');
        if (!empty($realIp)) {
            return $realIp;
        }
        
        // Fall back to remote address
        return $serverParams['REMOTE_ADDR'] ?? 'unknown';
    }
}