import { useState } from 'react'
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/20/solid'
import { PlayIcon, DocumentTextIcon, CogIcon } from '@heroicons/react/24/outline'

const Tools = () => {
  const [activeTab, setActiveTab] = useState('generator')
  const [expanded, setExpanded] = useState({})

  const toggleExpanded = (id) => {
    setExpanded(prev => ({ ...prev, [id]: !prev[id] }))
  }

  const examples = {
    simple: {
      title: 'Simple GET Request',
      description: 'Basic API health check',
      code: `project: API Health Check
is-http: true

====

GET /health HTTP/1.1
host: api.example.com

====

status-code equals: 200
response-body contains: ok`
    },
    post: {
      title: 'POST with JSON',
      description: 'Create a new user',
      code: `project: Create User
is-http: true

====

POST /users HTTP/1.1
host: api.example.com
content-type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}

====

status-code equals: 201
response-body.id exists: true
response-body.name equals: John Doe`
    },
    auth: {
      title: 'Authentication Chain',
      description: 'Login and use token',
      code: `-- auth.greq
project: Login
is-http: true

====

POST /auth/login HTTP/1.1
host: api.example.com
content-type: application/json

{
  "username": "user@example.com",
  "password": "secret123"
}

====

status-code equals: 200
response-body.token exists: true

---

-- protected.greq
depends-on: auth.greq
project: Protected Resource
is-http: true

====

GET /profile HTTP/1.1
host: api.example.com
authorization: Bearer $(dependency.response-body.token)

====

status-code equals: 200
response-body.username exists: true`
    }
  }

  return (
    <section id="tools" className="py-24 bg-gray-50">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Interactive Tools
          </h2>
          <p className="mt-4 text-lg leading-8 text-gray-600">
            Explore examples and learn by doing
          </p>
        </div>

        <div className="mx-auto mt-16 max-w-5xl">
          {/* Tab Navigation */}
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-8">
              <button
                onClick={() => setActiveTab('generator')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'generator'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <PlayIcon className="inline h-4 w-4 mr-2" />
                Examples
              </button>
              <button
                onClick={() => setActiveTab('syntax')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'syntax'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <DocumentTextIcon className="inline h-4 w-4 mr-2" />
                Syntax Reference
              </button>
              <button
                onClick={() => setActiveTab('config')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'config'
                    ? 'border-primary-500 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <CogIcon className="inline h-4 w-4 mr-2" />
                Configuration
              </button>
            </nav>
          </div>

          {/* Tab Content */}
          <div className="mt-8">
            {activeTab === 'generator' && (
              <div className="space-y-6">
                {Object.entries(examples).map(([key, example]) => (
                  <div key={key} className="border border-gray-200 rounded-lg overflow-hidden">
                    <button
                      onClick={() => toggleExpanded(key)}
                      className="w-full px-6 py-4 text-left bg-white hover:bg-gray-50 flex items-center justify-between"
                    >
                      <div>
                        <h3 className="text-lg font-semibold text-gray-900">{example.title}</h3>
                        <p className="text-sm text-gray-600 mt-1">{example.description}</p>
                      </div>
                      {expanded[key] ? (
                        <ChevronDownIcon className="h-5 w-5 text-gray-400" />
                      ) : (
                        <ChevronRightIcon className="h-5 w-5 text-gray-400" />
                      )}
                    </button>
                    {expanded[key] && (
                      <div className="px-6 pb-6 bg-gray-50">
                        <div className="code-block mt-4">
                          <pre className="overflow-x-auto">
                            <code className="text-sm whitespace-pre-wrap">{example.code}</code>
                          </pre>
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {activeTab === 'syntax' && (
              <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Greq File Structure</h3>
                <div className="space-y-6">
                  <div>
                    <h4 className="font-medium text-gray-900 mb-2">Header Properties</h4>
                    <div className="code-block">
                      <pre>
                        <code className="text-sm">{`project: Test Name
is-http: true
timeout: 5000
extends: base.greq
depends-on: auth.greq`}</code>
                      </pre>
                    </div>
                  </div>
                  <div>
                    <h4 className="font-medium text-gray-900 mb-2">HTTP Request</h4>
                    <div className="code-block">
                      <pre>
                        <code className="text-sm">{`POST /api/users HTTP/1.1
host: api.example.com
content-type: application/json

{"name": "John", "email": "john@example.com"}`}</code>
                      </pre>
                    </div>
                  </div>
                  <div>
                    <h4 className="font-medium text-gray-900 mb-2">Response Validation</h4>
                    <div className="code-block">
                      <pre>
                        <code className="text-sm">{`status-code equals: 201
response-body.id exists: true
headers.content-type contains: json
latency less-than: 1000`}</code>
                      </pre>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'config' && (
              <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Configuration Options</h3>
                <div className="overflow-hidden">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Property
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Description
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Example
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {[
                        { prop: 'project', desc: 'Test project name', example: 'project: User API Tests' },
                        { prop: 'is-http', desc: 'Use HTTP instead of HTTPS', example: 'is-http: true' },
                        { prop: 'timeout', desc: 'Request timeout in milliseconds', example: 'timeout: 5000' },
                        { prop: 'extends', desc: 'Inherit from base file', example: 'extends: base.greq' },
                        { prop: 'depends-on', desc: 'Execute dependency first', example: 'depends-on: auth.greq' },
                        { prop: 'number-of-retries', desc: 'Retry attempts on failure', example: 'number-of-retries: 3' }
                      ].map((row, idx) => (
                        <tr key={idx} className={idx % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
                            {row.prop}
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-900">
                            {row.desc}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-500">
                            {row.example}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  )
}

export default Tools