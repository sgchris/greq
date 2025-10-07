import { 
  CodeBracketIcon, 
  LinkIcon, 
  CheckCircleIcon, 
  CommandLineIcon,
  ClockIcon,
  ArrowPathIcon,
  RocketLaunchIcon,
  KeyIcon
} from '@heroicons/react/24/outline'

const Features = () => {
  const features = [
    {
      name: 'Template Inheritance',
      description: 'Extend base configurations and reuse common settings across multiple tests.',
      icon: CodeBracketIcon,
      example: `extends: base-config.greq
project: User API Test

====

GET /api/users HTTP/1.1

====

response-body contains: users`
    },
    {
      name: 'Request Dependencies',
      description: 'Chain requests together and use response data in subsequent tests.',
      icon: LinkIcon,
      example: `depends-on: auth.greq
project: Protected Resource

====

GET /api/protected HTTP/1.1
authorization: Bearer $(dependency.response-body.token)

====

status-code equals: 200`
    },
    {
      name: 'Set Environment Variables',
      description: 'Capture response data and set environment variables for use in subsequent requests.',
      icon: KeyIcon,
      example: `depends-on: login.greq
set-environment.AUTH_TOKEN: $(dependency.response-body.auth_token)
set-environment.USER_ID: $(dependency.response-body.user.id)

====

GET /api/users/$(environment.USER_ID) HTTP/1.1
authorization: Bearer $(environment.AUTH_TOKEN)

====

status-code equals: 200`
    },
    {
      name: 'Advanced Validation',
      description: 'Validate status codes, headers, response bodies, and JSON paths with flexible operators.',
      icon: CheckCircleIcon,
      example: `status-code equals: 200
headers.content-type contains: json
response-body.user.id exists: true
latency less-than: 1000
not response-body contains: error`
    },
    {
      name: 'Environment Variables',
      description: 'Use environment variables for dynamic configuration across different environments.',
      icon: CommandLineIcon,
      example: `GET /api/users HTTP/1.1
host: $(environment.API_HOST)
authorization: Bearer $(environment.API_TOKEN)

====

status-code equals: 200`
    },
    {
      name: 'Timeout Control',
      description: 'Set request timeouts and retry policies for robust testing.',
      icon: ClockIcon,
      example: `project: Timeout Test
timeout: 5000
number-of-retries: 3

====

GET /slow-endpoint HTTP/1.1
host: api.example.com`
    },
    {
      name: 'Dependency Failure Handling',
      description: 'Control how tests behave when dependencies fail with flexible failure policies.',
      icon: ArrowPathIcon,
      example: `depends-on: cleanup.greq
allow-dependency-failure: true
show-warnings: false

====

POST /users HTTP/1.1
content-type: application/json`
    },
    {
      name: 'Shell Command Execution',
      description: 'Execute shell commands before and after HTTP requests for setup, cleanup, and automation.',
      icon: RocketLaunchIcon,
      example: `depends-on: create-user.greq
execute-before: echo "Setting up test data"
execute-after: ./cleanup.sh $(dependency.response-body.id)

====

DELETE /users/$(dependency.response-body.id) HTTP/1.1
host: api.example.com

====

status-code equals: 200`
    }
  ]

  return (
    <section id="features" className="py-24 bg-white">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
            Powerful Features
          </h2>
          <p className="mt-4 text-lg leading-8 text-gray-600">
            Everything you need to build comprehensive API test suites
          </p>
        </div>
        
        <div className="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none">
          <dl className="grid max-w-xl grid-cols-1 gap-x-8 gap-y-16 lg:max-w-none lg:grid-cols-3">
            {features.map((feature) => (
              <div key={feature.name} className="flex flex-col">
                <dt className="flex items-center gap-x-3 text-base font-semibold leading-7 text-gray-900">
                  <feature.icon className="h-5 w-5 flex-none text-primary-500" />
                  {feature.name}
                </dt>
                <dd className="mt-4 flex flex-auto flex-col text-base leading-7 text-gray-600">
                  <p className="flex-auto">{feature.description}</p>
                  <div className="mt-6">
                    <div className="code-block">
                      <pre className="overflow-x-auto">
                        <code className="text-sm whitespace-pre-wrap">{feature.example}</code>
                      </pre>
                    </div>
                  </div>
                </dd>
              </div>
            ))}
          </dl>
        </div>
      </div>
    </section>
  )
}

export default Features