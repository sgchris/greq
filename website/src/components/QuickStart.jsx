import { useState } from 'react'
import { ClipboardIcon, CheckIcon } from '@heroicons/react/24/outline'

const QuickStart = () => {
  const [copied, setCopied] = useState('')

  const copyToClipboard = async (text, key) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(key)
      setTimeout(() => setCopied(''), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }

  const greqExample = `project: User API Test

====

POST /users HTTP/1.1
host: api.example.com
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

====

status-code equals: 200
or status-code equals: 201
headers.content-type equals: application/json`

  const terminalCommand = 'greq user-test.greq'

  return (
    <section id="quickstart" className="py-16 bg-gray-50">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Get Started in Seconds
          </h2>
          <p className="mt-4 text-lg leading-8 text-gray-600">
            Create your first API test with simple, readable syntax
          </p>
        </div>
        
        <div className="mt-12 grid gap-8 lg:grid-cols-2">
          {/* Code example */}
          <div>
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-lg font-semibold text-gray-900">user-test.greq</h3>
              <button
                onClick={() => copyToClipboard(greqExample, 'greq')}
                className="flex items-center gap-2 rounded-md bg-gray-800 px-3 py-1 text-sm text-white hover:bg-gray-700 transition-colors"
              >
                {copied === 'greq' ? (
                  <CheckIcon className="h-4 w-4" />
                ) : (
                  <ClipboardIcon className="h-4 w-4" />
                )}
                {copied === 'greq' ? 'Copied!' : 'Copy'}
              </button>
            </div>
            <div className="code-block relative">
              <pre className="overflow-x-auto">
                <code className="text-sm">{greqExample}</code>
              </pre>
            </div>
          </div>
          
          {/* Terminal output */}
          <div>
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-lg font-semibold text-gray-900">Run the test</h3>
              <button
                onClick={() => copyToClipboard(terminalCommand, 'cmd')}
                className="flex items-center gap-2 rounded-md bg-gray-800 px-3 py-1 text-sm text-white hover:bg-gray-700 transition-colors"
              >
                {copied === 'cmd' ? (
                  <CheckIcon className="h-4 w-4" />
                ) : (
                  <ClipboardIcon className="h-4 w-4" />
                )}
                {copied === 'cmd' ? 'Copied!' : 'Copy'}
              </button>
            </div>
            <div className="code-block">
              <pre>
                <code className="text-sm">
                  <span className="text-green-400">$</span> {terminalCommand}
                  {"\n\n"}
                  <span className="text-green-400">✓</span> user-test.greq
                  {"\n"}
                  <span className="text-gray-400">  Status: 201 (245ms)</span>
                  {"\n\n"}
                  <span className="text-green-400">Summary: All 1 tests passed</span>
                </code>
              </pre>
            </div>
            
            <div className="mt-6 space-y-4">
              <div className="flex items-start gap-3">
                <div className="flex-shrink-0 w-6 h-6 bg-green-100 rounded-full flex items-center justify-center">
                  <span className="text-green-600 text-sm font-bold">1</span>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-900">Create a .greq file</p>
                  <p className="text-sm text-gray-600">Define your API test with clear, readable syntax</p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <div className="flex-shrink-0 w-6 h-6 bg-green-100 rounded-full flex items-center justify-center">
                  <span className="text-green-600 text-sm font-bold">2</span>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-900">Run the test</p>
                  <p className="text-sm text-gray-600">Execute with a simple command and get instant feedback</p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <div className="flex-shrink-0 w-6 h-6 bg-green-100 rounded-full flex items-center justify-center">
                  <span className="text-green-600 text-sm font-bold">3</span>
                </div>
                <div>
                  <p className="text-sm font-medium text-gray-900">Validate results</p>
                  <p className="text-sm text-gray-600">Automatic validation with detailed pass/fail reporting</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

export default QuickStart