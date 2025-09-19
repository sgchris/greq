# Greq Website Specification

## Overview
A modern, clean website to introduce and showcase the Greq HTTP testing tool. The design follows the aesthetic of scalatut.greq.me and peegees.greq.me with white, clean, readable, and convenient styling.

## Technology Stack
- **Framework**: React 18+
- **Styling**: TailwindCSS
- **Build Tool**: Vite
- **Icons**: Heroicons
- **Fonts**: Google Fonts (Inter for body, JetBrains Mono for code)
- **Analytics**: Google Analytics 4 (gtag)
- **SEO**: React Helmet Async for meta tags

## Design Principles
- **Clean & Minimal**: White background, plenty of whitespace
- **Readable Typography**: Clear hierarchy, good contrast
- **Mobile-First**: Responsive design for all screen sizes
- **Accessible**: WCAG 2.1 AA compliance
- **Fast Loading**: Optimized images, lazy loading

## Page Structure

### 1. Header Section
```jsx
// Hero section with title and tagline
<header className="bg-white">
  <nav> // Navigation with logo and menu
  <section> // Hero content
    <h1>Greq 🚀</h1>
    <p>A powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters.</p>
    <div> // CTA buttons: "Get Started", "View on GitHub", "Download"
</header>
```

**Content**:
- **Title**: "Greq 🚀"
- **Tagline**: "A powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters."
- **Description**: "Write expressive HTTP tests, chain requests with dependencies, and validate responses with ease."
- **CTA Buttons**: 
  - "Get Started" (scroll to examples)
  - "View on GitHub" (external link)
  - "Download" (link to releases)

### 2. Quick Start Section
```jsx
// Basic example with syntax-highlighted code
<section className="py-16 bg-gray-50">
  <div className="container">
    <h2>Get Started in Seconds</h2>
    <div className="grid md:grid-cols-2 gap-8">
      <div> // Code example
      <div> // Terminal output
```

**Content**:
- **Basic Example**:
`user-test.greq` file contents:

```greq
project: User API Test

====

POST /users HTTP/1.1
host: api.example.com
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

====

statis-code equals: 200
or status-code equals: 201
headers.content-type equals: application/json

```

- **Terminal Command**: `greq user-test.greq`

### 3. Features Showcase

A table with 
- feature name
- feature description
- example 
(the example is a line in the header, content or footer sections)

**Features to Highlight**:

#### 3.1 Inheritance System
```greq
extends: base-config.greq
project: User API Test

====

GET /users/123 HTTP/1.1
// Inherits host and common headers

====

response-body.json.id equals: 123
```

#### 3.2 Smart Dependencies
```greq
depends-on: auth.greq
project: Protected Resource

====

GET /profile HTTP/1.1
authorization: Bearer $(dependency.response-body.token)

====

status-code equals: 200
```

#### 3.3 Environment Variables
```greq
project: Environment Test

====

POST /api/users HTTP/1.1
host: $(environment.api-host)
authorization: Bearer $(environment.api-token)

====

status-code equals: 201
```

#### 3.4 Advanced response evaluations
```greq
====

status-code equals: 200
or status-code equals: 201
response-body.json.users[0].name exists: true
not response-body contains: error
latency less-than: 1000
headers.content-type contains: json
```

#### 3.5 Dependency Failure Handling
```greq
depends-on: cleanup.greq
allow-dependency-failure: true
show-warnings: false

====
// Continues even if cleanup fails
```

#### 3.6 Verbose Debugging
```bash
greq --verbose api-test.greq
# Shows detailed request/response/error info
```

### 4. Interactive Tools Section
```jsx
// Links to additional resources
<section className="py-16 bg-indigo-50">
  <div className="container text-center">
    <h2>Explore More</h2>
    <div className="grid md:grid-cols-3 gap-8">
      // Tool cards
```

**Content**:
- **Documentation**: Complete guide with examples
- **Request Builder**: Interactive tool to build .greq files
- **Flow Designer**: Visual dependency chain builder
- **Examples Gallery**: Pre-built test scenarios

### 5. Newsletter Signup
```jsx
// Email subscription form
<section className="py-16 bg-white">
  <div className="container max-w-2xl text-center">
    <h2>Stay Updated</h2>
    <form> // Email input and subscribe button
```

**Content**:
- **Title**: "Stay Updated"
- **Description**: "Get notified about new features, examples, and best practices."
- **Form**: Email input with validation
- **Privacy**: "No spam, unsubscribe anytime"

### 6. Footer
```jsx
// Site footer with links and info
<footer className="bg-gray-900 text-white py-12">
  <div className="container">
    <div className="grid md:grid-cols-4 gap-8">
      // Footer sections
```

**Content**:
- **About**: Brief description and logo
- **Resources**: 
  - Documentation
  - GitHub Repository
  - Examples
  - Changelog
- **Community**:
  - Issues & Support
  - Discussions
  - Contributing Guide
- **Legal**:
  - License (MIT)
  - Privacy Policy

## SEO Requirements

### Meta Tags
```html
<title>Greq - Powerful API Testing Tool | HTTP Request Testing Made Easy</title>
<meta name="description" content="Greq is a a powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters.">
<meta name="keywords" content="API testing, HTTP testing, CLI tool, REST API, request testing, automation">

<!-- Open Graph -->
<meta property="og:title" content="Greq - Powerful API Testing Tool">
<meta property="og:description" content="Write expressive HTTP tests with templates, dependencies, and response evaluations.">
<meta property="og:type" content="website">
<meta property="og:url" content="https://greq.dev">
<meta property="og:image" content="https://greq.dev/og-image.png">

<!-- Twitter Cards -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="Greq - Powerful API Testing Tool">
<meta name="twitter:description" content="Write expressive HTTP tests with templates, dependencies, and response evaluations.">
<meta name="twitter:image" content="https://greq.dev/twitter-image.png">
```

### Structured Data
```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Greq",
  "description": "A powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters.",
  "url": "https://greq.dev",
  "downloadUrl": "https://github.com/sgchris/greq/releases",
  "operatingSystem": "Windows, macOS, Linux",
  "programmingLanguage": "Rust",
  "license": "MIT"
}
```

## Responsive Design

### Breakpoints
- **Mobile**: 0-640px
- **Tablet**: 641-1024px
- **Desktop**: 1025px+

### Key Responsive Elements
- **Navigation**: Hamburger menu on mobile
- **Code Blocks**: Horizontal scroll on small screens
- **Feature Grid**: Stacked on mobile, 2-col on tablet, 3-col on desktop
- **Hero**: Centered text on mobile, split layout on desktop

## Performance Requirements
- **Core Web Vitals**: 
  - LCP < 2.5s
  - FID < 100ms
  - CLS < 0.1
- **Lighthouse Score**: 90+ in all categories
- **Image Optimization**: WebP format with fallbacks
- **Code Splitting**: Lazy load non-critical components

## Accessibility Features
- **Semantic HTML**: Proper heading hierarchy
- **ARIA Labels**: Screen reader support
- **Color Contrast**: WCAG AA compliance (4.5:1 ratio)
- **Keyboard Navigation**: Full keyboard accessibility
- **Focus Indicators**: Clear focus states

## Google Analytics Setup
```html
<!-- Google Analytics 4 -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

**Events to Track**:
- Page views
- Documentation clicks
- GitHub stars
- Newsletter signups
- Download attempts

## Content Guidelines

### Tone & Voice
- **Professional yet approachable**
- **Technical but accessible**
- **Concise and clear**
- **Action-oriented**

### Code Examples
- **Syntax highlighting** using Prism.js or similar
- **Copy-to-clipboard** functionality
- **Real-world scenarios**
- **Progressive complexity**

### Visual Elements
- **Clean icons** from Heroicons
- **Consistent color scheme**: 
  - Primary: Indigo (#4F46E5)
  - Success: Green (#10B981)
  - Warning: Amber (#F59E0B)
  - Error: Red (#EF4444)
- **Subtle shadows and borders**
- **Smooth animations and transitions**

## File Structure
```
website/
├── public/
│   ├── favicon.ico
│   ├── og-image.png
│   └── robots.txt
├── src/
│   ├── components/
│   │   ├── Header.jsx
│   │   ├── Hero.jsx
│   │   ├── QuickStart.jsx
│   │   ├── Features.jsx
│   │   ├── Tools.jsx
│   │   ├── Newsletter.jsx
│   │   └── Footer.jsx
│   ├── styles/
│   │   └── index.css
│   ├── App.jsx
│   └── main.jsx
├── package.json
├── vite.config.js
├── tailwind.config.js
└── README.md
```

## Implementation Priority
1. **Setup & Basic Structure** (React + Vite + TailwindCSS)
2. **Hero Section** (Title, tagline, CTAs)
3. **Quick Start** (Basic example with syntax highlighting)
4. **Features Showcase** (Core functionality examples)
5. **Interactive Tools** (Links and descriptions)
6. **Newsletter** (Email subscription form)
7. **Footer** (Links and legal info)
8. **SEO & Analytics** (Meta tags, structured data, gtag)
9. **Performance Optimization** (Image optimization, code splitting)
10. **Accessibility Audit** (WCAG compliance testing)

## Success Metrics
- **User Engagement**: Time on site, scroll depth
- **Conversions**: GitHub stars, documentation visits
- **Performance**: Core Web Vitals scores
- **SEO**: Search ranking for "API testing CLI"
- **Accessibility**: Lighthouse accessibility score 95+
