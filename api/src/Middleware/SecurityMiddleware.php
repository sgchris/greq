<?php

namespace Greq\Api\Middleware;

use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;
use Slim\Psr7\Factory\ResponseFactory;

/**
 * Security middleware for XSS protection, bot detection, and security headers
 */
class SecurityMiddleware implements MiddlewareInterface
{
    private LoggerInterface $logger;
    private ResponseFactory $responseFactory;

    public function __construct(LoggerInterface $logger)
    {
        $this->logger = $logger;
        $this->responseFactory = new ResponseFactory();
    }

    public function process(Request $request, RequestHandler $handler): Response
    {
        // Bot detection
        if ($this->isBot($request)) {
            $this->logger->warning('Bot request blocked', [
                'user_agent' => $request->getHeaderLine('User-Agent'),
                'ip' => $this->getClientIp($request)
            ]);
            
            $response = $this->responseFactory->createResponse(403);
            $response->getBody()->write(json_encode([
                'error' => 'Access denied',
                'message' => 'Bot requests are not allowed'
            ]));
            
            return $response->withHeader('Content-Type', 'application/json');
        }

        $response = $handler->handle($request);
        
        // Add security headers
        $response = $this->addSecurityHeaders($response);
        
        return $response;
    }

    /**
     * Detect if request is from a bot
     */
    private function isBot(Request $request): bool
    {
        $userAgent = strtolower($request->getHeaderLine('User-Agent'));
        
        if (empty($userAgent)) {
            return true; // No user agent = likely bot
        }
        
        // Common bot patterns
        $botPatterns = [
            'bot', 'crawler', 'spider', 'scraper', 'curl', 'wget', 
            'python', 'java', 'perl', 'ruby', 'go-http', 'libwww',
            'axios', 'postman', 'insomnia', 'httpie'
        ];
        
        foreach ($botPatterns as $pattern) {
            if (strpos($userAgent, $pattern) !== false) {
                return true;
            }
        }
        
        // Check if it looks like a real browser
        $browserPatterns = ['mozilla', 'webkit', 'gecko', 'chrome', 'safari', 'firefox', 'edge'];
        $hasBrowserPattern = false;
        
        foreach ($browserPatterns as $pattern) {
            if (strpos($userAgent, $pattern) !== false) {
                $hasBrowserPattern = true;
                break;
            }
        }
        
        return !$hasBrowserPattern;
    }

    /**
     * Add security headers to response
     */
    private function addSecurityHeaders(Response $response): Response
    {
        return $response
            ->withHeader('X-Content-Type-Options', 'nosniff')
            ->withHeader('X-Frame-Options', 'DENY')
            ->withHeader('X-XSS-Protection', '1; mode=block')
            ->withHeader('Referrer-Policy', 'strict-origin-when-cross-origin')
            ->withHeader('Content-Security-Policy', "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:;");
    }

    /**
     * Sanitize input to prevent XSS
     */
    public static function sanitizeInput(string $input): string
    {
        return htmlspecialchars(trim($input), ENT_QUOTES, 'UTF-8');
    }

    /**
     * Validate email address
     */
    public static function validateEmail(string $email): bool
    {
        return filter_var($email, FILTER_VALIDATE_EMAIL) !== false;
    }

    private function getClientIp(Request $request): string
    {
        $serverParams = $request->getServerParams();
        
        $forwardedFor = $request->getHeaderLine('X-Forwarded-For');
        if (!empty($forwardedFor)) {
            $ips = explode(',', $forwardedFor);
            return trim($ips[0]);
        }
        
        $realIp = $request->getHeaderLine('X-Real-IP');
        if (!empty($realIp)) {
            return $realIp;
        }
        
        return $serverParams['REMOTE_ADDR'] ?? 'unknown';
    }
}