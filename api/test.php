<?php
/**
 * Simple test script for Greq API endpoints
 */

require __DIR__ . '/vendor/autoload.php';

class ApiTester
{
    private string $baseUrl;
    private string $adminPassword;

    public function __construct(string $baseUrl = 'http://localhost:8080', string $adminPassword = 'greq2024!')
    {
        $this->baseUrl = rtrim($baseUrl, '/');
        $this->adminPassword = $adminPassword;
    }

    public function runTests(): void
    {
        echo "🚀 Greq API Test Suite\n";
        echo "========================\n\n";

        $this->testHealthEndpoint();
        $this->testSubscribeEndpoint();
        $this->testSubscribersEndpoint();
        $this->testDuplicateSubscription();
        $this->testInvalidEmail();

        echo "\n✅ All tests completed!\n";
    }

    private function testHealthEndpoint(): void
    {
        echo "1️⃣ Testing health endpoint...\n";
        
        $response = $this->makeRequest('GET', '/api/health');
        
        if ($response['http_code'] === 200) {
            $data = json_decode($response['body'], true);
            if ($data && isset($data['status']) && $data['status'] === 'healthy') {
                echo "   ✅ Health check passed\n\n";
            } else {
                echo "   ❌ Health check failed - invalid response\n\n";
            }
        } else {
            echo "   ❌ Health check failed - HTTP {$response['http_code']}\n\n";
        }
    }

    private function testSubscribeEndpoint(): void
    {
        echo "2️⃣ Testing subscription endpoint...\n";
        
        $testEmail = 'test' . time() . '@example.com';
        $postData = json_encode(['email' => $testEmail]);
        
        $response = $this->makeRequest('POST', '/api/subscribe', $postData, [
            'Content-Type: application/json'
        ]);
        
        if ($response['http_code'] === 201) {
            $data = json_decode($response['body'], true);
            if ($data && $data['success'] === true) {
                echo "   ✅ Subscription successful for {$testEmail}\n\n";
            } else {
                echo "   ❌ Subscription failed - invalid response\n\n";
            }
        } else {
            echo "   ❌ Subscription failed - HTTP {$response['http_code']}\n";
            echo "   Response: {$response['body']}\n\n";
        }
    }

    private function testSubscribersEndpoint(): void
    {
        echo "3️⃣ Testing subscribers endpoint (with auth)...\n";
        
        $credentials = base64_encode("admin:{$this->adminPassword}");
        $response = $this->makeRequest('GET', '/api/subscribers', null, [
            "Authorization: Basic {$credentials}"
        ]);
        
        if ($response['http_code'] === 200) {
            if (strpos($response['body'], '<title>Greq Newsletter Subscribers</title>') !== false) {
                echo "   ✅ Subscribers page loaded successfully\n\n";
            } else {
                echo "   ❌ Subscribers endpoint failed - unexpected content\n\n";
            }
        } else {
            echo "   ❌ Subscribers endpoint failed - HTTP {$response['http_code']}\n\n";
        }
    }

    private function testDuplicateSubscription(): void
    {
        echo "4️⃣ Testing duplicate subscription handling...\n";
        
        $testEmail = 'duplicate@example.com';
        $postData = json_encode(['email' => $testEmail]);
        
        // First subscription
        $response1 = $this->makeRequest('POST', '/api/subscribe', $postData, [
            'Content-Type: application/json'
        ]);
        
        // Second subscription (should fail)
        $response2 = $this->makeRequest('POST', '/api/subscribe', $postData, [
            'Content-Type: application/json'
        ]);
        
        if ($response2['http_code'] === 409) {
            echo "   ✅ Duplicate subscription properly rejected\n\n";
        } else {
            echo "   ❌ Duplicate subscription handling failed - HTTP {$response2['http_code']}\n\n";
        }
    }

    private function testInvalidEmail(): void
    {
        echo "5️⃣ Testing invalid email handling...\n";
        
        $postData = json_encode(['email' => 'not-an-email']);
        
        $response = $this->makeRequest('POST', '/api/subscribe', $postData, [
            'Content-Type: application/json'
        ]);
        
        if ($response['http_code'] === 400) {
            echo "   ✅ Invalid email properly rejected\n\n";
        } else {
            echo "   ❌ Invalid email handling failed - HTTP {$response['http_code']}\n\n";
        }
    }

    private function makeRequest(string $method, string $endpoint, ?string $data = null, array $headers = []): array
    {
        $url = $this->baseUrl . $endpoint;
        
        $ch = curl_init();
        curl_setopt_array($ch, [
            CURLOPT_URL => $url,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_TIMEOUT => 10,
            CURLOPT_CUSTOMREQUEST => $method,
            CURLOPT_HTTPHEADER => $headers,
        ]);
        
        if ($data !== null) {
            curl_setopt($ch, CURLOPT_POSTFIELDS, $data);
        }
        
        $response = curl_exec($ch);
        $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);
        
        return [
            'body' => $response,
            'http_code' => $httpCode
        ];
    }
}

// Run tests if script is executed directly
if (basename(__FILE__) === basename($_SERVER['SCRIPT_NAME'])) {
    $tester = new ApiTester();
    $tester->runTests();
}