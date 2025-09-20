<?php
/**
 * Security Test Script for Greq Newsletter System
 * 
 * Tests the rate limiting and security features.
 * Run from command line: php test_security.php
 */

echo "Greq Newsletter Security Test\n";
echo "==============================\n\n";

$baseUrl = 'http://localhost:8080';
$testEmail = 'test' . time() . '@example.com';

function makeRequest($url, $data = null) {
    $ch = curl_init();
    curl_setopt($ch, CURLOPT_URL, $url);
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($ch, CURLOPT_HEADER, true);
    curl_setopt($ch, CURLOPT_TIMEOUT, 10);
    
    if ($data) {
        curl_setopt($ch, CURLOPT_POST, true);
        curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode($data));
        curl_setopt($ch, CURLOPT_HTTPHEADER, ['Content-Type: application/json']);
    }
    
    $response = curl_exec($ch);
    $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
    curl_close($ch);
    
    return ['code' => $httpCode, 'response' => $response];
}

// Test 1: API Information
echo "1. Testing API Information Endpoint\n";
$result = makeRequest($baseUrl . '/');
echo "   Status Code: " . $result['code'] . "\n";
if ($result['code'] === 200) {
    echo "   ✓ API endpoint working\n";
} else {
    echo "   ✗ API endpoint failed\n";
}
echo "\n";

// Test 2: Security Headers
echo "2. Testing Security Headers\n";
if (strpos($result['response'], 'X-Content-Type-Options') !== false) {
    echo "   ✓ X-Content-Type-Options header present\n";
} else {
    echo "   ✗ X-Content-Type-Options header missing\n";
}

if (strpos($result['response'], 'X-Frame-Options') !== false) {
    echo "   ✓ X-Frame-Options header present\n";
} else {
    echo "   ✗ X-Frame-Options header missing\n";
}

if (strpos($result['response'], 'Content-Security-Policy') !== false) {
    echo "   ✓ Content-Security-Policy header present\n";
} else {
    echo "   ✗ Content-Security-Policy header missing\n";
}
echo "\n";

// Test 3: Valid Newsletter Subscription
echo "3. Testing Valid Newsletter Subscription\n";
$result = makeRequest($baseUrl . '/subscribe.php', ['email' => $testEmail]);
echo "   Status Code: " . $result['code'] . "\n";
if ($result['code'] === 201) {
    echo "   ✓ Newsletter subscription successful\n";
} else {
    echo "   ✗ Newsletter subscription failed\n";
    echo "   Response: " . substr($result['response'], -200) . "\n";
}
echo "\n";

// Test 4: Duplicate Subscription (should fail)
echo "4. Testing Duplicate Newsletter Subscription\n";
$result = makeRequest($baseUrl . '/subscribe.php', ['email' => $testEmail]);
echo "   Status Code: " . $result['code'] . "\n";
if ($result['code'] === 409) {
    echo "   ✓ Duplicate subscription properly rejected\n";
} else {
    echo "   ✗ Duplicate subscription not handled correctly\n";
}
echo "\n";

// Test 5: Rate Limiting
echo "5. Testing Rate Limiting (making 7 requests rapidly)\n";
$rateLimitHit = false;
for ($i = 1; $i <= 7; $i++) {
    $result = makeRequest($baseUrl . '/subscribe.php', ['email' => "test{$i}" . time() . "@example.com"]);
    echo "   Request $i: Status Code " . $result['code'] . "\n";
    
    if ($result['code'] === 429) {
        echo "   ✓ Rate limit triggered at request $i\n";
        $rateLimitHit = true;
        break;
    }
    
    // Small delay to avoid overwhelming the server
    usleep(100000); // 0.1 second
}

if (!$rateLimitHit) {
    echo "   ! Rate limit not triggered (may need adjustment or more requests)\n";
}
echo "\n";

// Test 6: Invalid Email
echo "6. Testing Invalid Email Validation\n";
$result = makeRequest($baseUrl . '/subscribe.php', ['email' => 'invalid-email']);
echo "   Status Code: " . $result['code'] . "\n";
if ($result['code'] === 400) {
    echo "   ✓ Invalid email properly rejected\n";
} else {
    echo "   ✗ Invalid email validation failed\n";
}
echo "\n";

// Test 7: Admin Access (should require authentication)
echo "7. Testing Admin Access Security\n";
$result = makeRequest($baseUrl . '/admin.php');
echo "   Status Code: " . $result['code'] . "\n";
if ($result['code'] === 401) {
    echo "   ✓ Admin access properly protected\n";
} else {
    echo "   ✗ Admin access not properly protected\n";
}
echo "\n";

echo "Security Test Complete!\n";
echo "\nNote: For HTTPS enforcement testing, deploy to a production server with SSL.\n";
echo "Rate limiting uses a 1-hour window, so limits may reset between test runs.\n";
?>