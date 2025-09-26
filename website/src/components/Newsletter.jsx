import { useState } from 'react'
import { CheckIcon } from '@heroicons/react/20/solid'
import { EnvelopeIcon } from '@heroicons/react/24/outline'

const Newsletter = () => {
  const [email, setEmail] = useState('')
  const [status, setStatus] = useState('') // 'loading', 'success', 'error'

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!email || !email.includes('@')) {
      setStatus('error')
      return
    }
    
    setStatus('loading')
    
    try {
      const response = await fetch('http://localhost:8080/subscribe.php', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        body: JSON.stringify({ email })
      })
      
      if (response.ok) {
        setStatus('success')
        setEmail('')
        
        // Reset success message after 3 seconds
        setTimeout(() => setStatus(''), 3000)
      } else {
        // Handle specific error messages from the server
        if (response.status === 409) {
          setStatus('already-subscribed')
        } else {
          setStatus('error')
        }
        setTimeout(() => setStatus(''), 3000)
      }
    } catch (error) {
      console.error('Subscription error:', error)
      setStatus('error')
      setTimeout(() => setStatus(''), 3000)
    }
  }

  return (
    <section className="py-16 bg-primary-600">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Stay Updated
          </h2>
          <p className="mt-4 text-lg leading-8 text-white">
            Get notified about new features, updates, and API testing best practices.
          </p>
          {/*
          <form onSubmit={handleSubmit} className="mt-8 flex flex-col sm:flex-row gap-4 justify-center">
            <div className="flex-1 max-w-md relative">
              <label htmlFor="email" className="sr-only">
                Email address
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <EnvelopeIcon className="h-5 w-5 text-gray-400" />
                </div>
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="Enter your email"
                  className="block w-full pl-10 pr-3 py-3 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-2 focus:ring-white focus:border-white"
                  disabled={status === 'loading'}
                />
              </div>
            </div>
            <button
              type="submit"
              disabled={status === 'loading' || status === 'success'}
              className="px-6 py-3 bg-white text-primary-600 font-semibold rounded-md hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors min-w-[120px] flex items-center justify-center"
            >
              {status === 'loading' && (
                <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-primary-600" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
              )}
              {status === 'success' && <CheckIcon className="h-4 w-4 mr-2" />}
              {status === 'loading' ? 'Subscribing...' : status === 'success' ? 'Subscribed!' : 'Subscribe'}
            </button>
          </form>
          
          {status === 'error' && (
            <p className="mt-3 text-sm text-red-200">
              Please enter a valid email address or try again later.
            </p>
          )}
          
          {status === 'already-subscribed' && (
            <p className="mt-3 text-sm text-yellow-200">
              This email is already subscribed to our newsletter.
            </p>
          )}
          
          <p className="mt-4 text-sm text-white">
            No spam, unsubscribe at any time. We respect your privacy.
          </p>
           */}
        </div>
      </div>
    </section>
  )
}

export default Newsletter