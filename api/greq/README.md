# 🧪 Greq API Test Suite

This directory contains comprehensive `.greq` files to test the Greq Newsletter API service.

## 📁 Test Files Overview

### Core API Tests
- **`01-health-check.greq`** - Tests the `/api/health` endpoint for service status
- **`02-subscribe-valid.greq`** - Tests valid email subscription via `/api/subscribe`
- **`03-invalid-email.greq`** - Tests rejection of invalid email formats
- **`04-duplicate-subscription.greq`** - Tests duplicate subscription handling (depends on #2)
- **`05-subscribers-dashboard.greq`** - Tests authenticated access to `/api/subscribers`

### Edge Case Tests  
- **`07-missing-email.greq`** - Tests missing email field validation
- **`08-unauthorized-access.greq`** - Tests unauthorized access to protected endpoints
- **`09-invalid-json.greq`** - Tests malformed JSON request handling

### Security & CORS Tests
- **`10-cors-security.greq`** - Tests CORS headers and security headers

### Comprehensive Tests
- **`06-workflow-test.greq`** - Complete workflow test with multiple steps
- **`base-config.greq`** - Base configuration for shared settings

## 🚀 Running the Tests

Make sure the API server is running:
```bash
cd api
php -S localhost:8080 -t public/
```

Then run the Greq tests:
```bash
# Run individual test
greq 01-health-check.greq

# Run all tests in order  
greq 01-health-check.greq 02-subscribe-valid.greq 03-invalid-email.greq 04-duplicate-subscription.greq 05-subscribers-dashboard.greq

# Run comprehensive workflow
greq 06-workflow-test.greq

# Test security features
greq 10-cors-security.greq
```

## 🔍 Test Coverage

### ✅ Endpoints Tested
- `GET /api/health` - Health check
- `POST /api/subscribe` - Newsletter subscription  
- `GET /api/subscribers` - Subscribers dashboard (auth required)
- `OPTIONS /api/subscribe` - CORS preflight

### ✅ Security Features Tested
- **Email Validation** - Valid/invalid email format handling
- **Authentication** - HTTP Basic auth for protected endpoints  
- **CORS** - Cross-origin request handling
- **Rate Limiting** - (Implicit via request patterns)
- **Security Headers** - XSS protection, frame options, content type
- **Input Validation** - JSON parsing, required fields
- **Duplicate Prevention** - Duplicate email subscription handling

### ✅ Error Scenarios Tested
- Invalid email format (400)
- Missing email field (400)  
- Malformed JSON (400)
- Duplicate subscription (409)
- Unauthorized access (401)

### ✅ Success Scenarios Tested
- Healthy service status (200)
- Successful subscription (201)
- Dashboard access with auth (200)
- CORS preflight response (200)

## 📊 Expected Results

| Test File | Expected Status | Key Assertions |
|-----------|----------------|----------------|
| `01-health-check.greq` | 200 | Service operational, security settings |
| `02-subscribe-valid.greq` | 201 | Successful subscription response |
| `03-invalid-email.greq` | 400 | Invalid email rejection |
| `04-duplicate-subscription.greq` | 409 | Duplicate email rejection |
| `05-subscribers-dashboard.greq` | 200 | HTML dashboard with subscriber data |
| `06-workflow-test.greq` | Mixed | Complete API workflow validation |
| `07-missing-email.greq` | 400 | Required field validation |
| `08-unauthorized-access.greq` | 401 | Authentication required |
| `09-invalid-json.greq` | 400 | JSON parsing error |
| `10-cors-security.greq` | 200 | CORS and security headers |

## 🔐 Authentication Details

For authenticated endpoints:
- **Username**: `admin`
- **Password**: `greq2024!`
- **Authorization Header**: `Basic YWRtaW46Z3JlcTIwMjQh`

## 📝 Notes

- Tests #4 depends on #2 (duplicate subscription test needs initial subscription)
- The workflow test (#6) is comprehensive and tests multiple scenarios in sequence
- Security headers are verified in multiple tests
- Base configuration is shared via `base-config.greq`
- All tests include latency assertions for performance validation

## 🎯 Test Philosophy

These tests validate:
1. **Functionality** - All endpoints work as expected
2. **Security** - Authentication, validation, and security headers
3. **Error Handling** - Proper error responses and status codes  
4. **Performance** - Response time requirements
5. **Integration** - End-to-end workflows and dependencies
6. **Standards** - HTTP compliance and API best practices

Run these tests regularly to ensure the API service maintains quality and security! 🚀