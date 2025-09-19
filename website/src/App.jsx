import { HelmetProvider } from 'react-helmet-async'
import Header from './components/Header'
import Hero from './components/Hero'
import QuickStart from './components/QuickStart'
import Features from './components/Features'
import Tools from './components/Tools'
import Newsletter from './components/Newsletter'
import Footer from './components/Footer'
import SEO from './components/SEO'

function App() {
  return (
    <HelmetProvider>
      <div className="min-h-screen bg-white">
        <SEO />
        <Header />
        <Hero />
        <QuickStart />
        <Features />
        <Tools />
        <Newsletter />
        <Footer />
      </div>
    </HelmetProvider>
  )
}

export default App
