<?php

namespace Greq\Api\Middleware;

use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;
use Slim\Psr7\Factory\ResponseFactory;

/**
 * Authentication middleware for password-protected endpoints
 */
class AuthMiddleware implements MiddlewareInterface
{
    private string $adminPassword;
    private LoggerInterface $logger;
    private ResponseFactory $responseFactory;

    public function __construct(string $adminPassword, LoggerInterface $logger)
    {
        $this->adminPassword = $adminPassword;
        $this->logger = $logger;
        $this->responseFactory = new ResponseFactory();
    }

    public function process(Request $request, RequestHandler $handler): Response
    {
        $authHeader = $request->getHeaderLine('Authorization');
        
        if (empty($authHeader)) {
            return $this->requireAuthentication();
        }
        
        // Parse Basic Auth
        if (!str_starts_with($authHeader, 'Basic ')) {
            return $this->requireAuthentication();
        }
        
        $credentials = base64_decode(substr($authHeader, 6));
        $parts = explode(':', $credentials, 2);
        
        if (count($parts) !== 2) {
            return $this->requireAuthentication();
        }
        
        [, $password] = $parts;
        
        if ($password !== $this->adminPassword) {
            $this->logger->warning('Authentication failed', [
                'ip' => $this->getClientIp($request),
                'user_agent' => $request->getHeaderLine('User-Agent')
            ]);
            return $this->requireAuthentication();
        }
        
        $this->logger->info('Authentication successful', [
            'ip' => $this->getClientIp($request)
        ]);
        
        return $handler->handle($request);
    }

    private function requireAuthentication(): Response
    {
        $response = $this->responseFactory->createResponse(401);
        return $response
            ->withHeader('WWW-Authenticate', 'Basic realm="Greq API Admin"')
            ->withHeader('Content-Type', 'application/json');
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
}