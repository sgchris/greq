# Greq Website

A modern, responsive website showcasing the Greq API testing tool built with React, Vite, and TailwindCSS.

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## 🏗️ Built With

- **React 18** - Modern UI framework
- **Vite** - Fast build tool and dev server
- **TailwindCSS** - Utility-first CSS framework
- **Heroicons** - Beautiful SVG icons
- **Radix UI Icons** - Additional icon set
- **React Helmet Async** - SEO optimization

## 📁 Project Structure

```
src/
├── components/          # React components
│   ├── SEO.jsx         # SEO metadata and structured data
│   ├── Header.jsx      # Navigation header
│   ├── Hero.jsx        # Hero section with CTA
│   ├── QuickStart.jsx  # Getting started section
│   ├── Features.jsx    # Feature showcase
│   ├── Tools.jsx       # Interactive examples and docs
│   ├── Newsletter.jsx  # Email signup
│   └── Footer.jsx      # Footer with links
├── App.jsx             # Main app component
├── index.css           # Global styles and Tailwind imports
└── main.jsx            # App entry point
```

## 🎨 Design System

- **Primary Colors**: Blue-based palette (`primary-*`)
- **Typography**: Inter font family with Tailwind typography classes
- **Components**: Consistent button styles, code blocks, and form elements
- **Responsive**: Mobile-first design with Tailwind responsive utilities

## 🔧 Customization

### Colors
Edit `tailwind.config.js` to customize the color palette:

```js
theme: {
  extend: {
    colors: {
      primary: {
        // Your custom colors
      }
    }
  }
}
```

### Components
All components are located in `src/components/` and use Tailwind classes for styling.

### Content
Update the content directly in the component files or extract to a separate content/data file for easier management.

## 📱 Features

- **Responsive Design**: Optimized for mobile, tablet, and desktop
- **SEO Optimized**: Meta tags, structured data, and semantic HTML
- **Interactive Examples**: Expandable code examples and syntax reference
- **Newsletter Signup**: Email collection with validation
- **Social Links**: GitHub, Twitter, Discord integration
- **Performance**: Optimized with Vite for fast loading

## 🚀 Newsletter Subscription

The website includes a functional newsletter subscription system with PHP backend:

### Setup Newsletter API

1. **Configure the system**:
   ```bash
   cd public
   cp config.template.php config.php
   # Edit config.php and set a strong admin password
   ```

2. **Start the PHP server** (in a separate terminal):
   ```bash
   php -S localhost:8080
   ```
   
   Or use the provided scripts:
   - Windows: `public/start-server.bat`
   - Unix/Mac: `public/start-server.sh`

3. **The React app will automatically connect** to the API at `http://localhost:8080`

4. **View subscribers** at `http://localhost:8080/admin.php` 
   - Default credentials: `admin` / `greq2025!secure`
   - **Change the password in config.php for production!**

### API Features

- **SQLite Database**: Automatic database creation and table setup
- **Email Validation**: Server-side email format validation
- **Duplicate Prevention**: Prevents duplicate subscriptions
- **CORS Support**: Configured for greq.me domain and subdomains
- **Admin Interface**: Password-protected subscriber management
- **Error Handling**: Comprehensive error responses
- **Security**: HTTP Basic Authentication for admin panel
- **Configuration**: Centralized config file for easy deployment

## 🚀 Deployment

The website can be deployed to any static hosting service:

```bash
# Build for production
npm run build

# The `dist` folder contains the built website
```

Popular deployment options:
- **Vercel**: `vercel deploy`
- **Netlify**: Drag and drop the `dist` folder
- **GitHub Pages**: Push to a branch and enable Pages
- **Cloudflare Pages**: Connect your repository

For production deployment of the newsletter API, deploy the `public/` folder to a PHP hosting service.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test the website locally
5. Submit a pull request

## 📄 License

This website is part of the Greq project. See the main project's LICENSE file for details.+ Vite

This template provides a minimal setup to get React working in Vite with HMR and some ESLint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Babel](https://babeljs.io/) for Fast Refresh
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/) for Fast Refresh

## Expanding the ESLint configuration

If you are developing a production application, we recommend using TypeScript with type-aware lint rules enabled. Check out the [TS template](https://github.com/vitejs/vite/tree/main/packages/create-vite/template-react-ts) for information on how to integrate TypeScript and [`typescript-eslint`](https://typescript-eslint.io) in your project.
