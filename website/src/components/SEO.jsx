import { Helmet } from 'react-helmet-async'

const SEO = () => {
  return (
    <Helmet>
      <title>Greq - Powerful API Testing Tool | HTTP Request Testing Made Easy</title>
      <meta name="description" content="Greq is a powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters." />
      <meta name="keywords" content="API testing, HTTP testing, CLI tool, REST API, request testing, automation" />
      
      {/* Open Graph */}
      <meta property="og:title" content="Greq - Powerful API Testing Tool" />
      <meta property="og:description" content="Write expressive HTTP tests with templates, dependencies, and response evaluations." />
      <meta property="og:type" content="website" />
      <meta property="og:url" content="https://greq.dev" />
      <meta property="og:image" content="https://greq.dev/og-image.png" />
      
      {/* Twitter Cards */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content="Greq - Powerful API Testing Tool" />
      <meta name="twitter:description" content="Write expressive HTTP tests with templates, dependencies, and response evaluations." />
      <meta name="twitter:image" content="https://greq.dev/twitter-image.png" />
      
      {/* Structured Data */}
      <script type="application/ld+json">
        {JSON.stringify({
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          "name": "Greq",
          "description": "A powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters.",
          "url": "https://greq.dev",
          "downloadUrl": "https://github.com/sgchris/greq/releases",
          "operatingSystem": "Windows, macOS, Linux",
          "programmingLanguage": "Rust",
          "license": "MIT"
        })}
      </script>
      
      {/* Google Analytics 4 */}
      <script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
      <script>
        {`
          window.dataLayer = window.dataLayer || [];
          function gtag(){dataLayer.push(arguments);}
          gtag('js', new Date());
          gtag('config', 'GA_MEASUREMENT_ID');
        `}
      </script>
      
      {/* Google Fonts Preconnect */}
      <link rel="preconnect" href="https://fonts.googleapis.com" />
      <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
    </Helmet>
  )
}

export default SEO