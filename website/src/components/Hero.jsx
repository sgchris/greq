import { ChevronRightIcon } from '@heroicons/react/20/solid'
import { StarIcon, CodeBracketIcon, CommandLineIcon } from '@heroicons/react/24/outline'

const Hero = () => {
  return (
    <div className="relative isolate px-6 pt-14 lg:px-8">
      <div className="mx-auto max-w-2xl py-32 sm:py-48 lg:py-56">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight text-gray-900 sm:text-6xl">
            Greq 🚀
          </h1>
          <p className="mt-6 text-lg leading-8 text-gray-600">
            A powerful APIs testing tool with templates, dependencies, response evaluation and dynamic parameters.
          </p>
          <p className="mt-2 text-base leading-7 text-gray-500">
            Write expressive HTTP tests, chain requests with dependencies, and validate responses with ease.
          </p>
          <div className="mt-10 flex items-center justify-center gap-x-6">
            <a
              href="#quickstart"
              className="btn-primary"
            >
              Get Started
            </a>
            <a
              href="https://github.com/sgchris/greq"
              className="btn-secondary group"
              target="_blank"
              rel="noopener noreferrer"
            >
              View on GitHub
              <ChevronRightIcon className="ml-2 h-4 w-4 group-hover:translate-x-1 transition-transform" />
            </a>
          </div>
          
          {/* Feature highlights */}
          <div className="mt-16 grid grid-cols-1 gap-8 sm:grid-cols-3">
            <div className="flex flex-col items-center">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary-50">
                <CodeBracketIcon className="h-6 w-6 text-primary-500" />
              </div>
              <h3 className="mt-4 text-sm font-semibold text-gray-900">Template System</h3>
              <p className="mt-2 text-sm text-gray-600">Inheritance and dependencies</p>
            </div>
            <div className="flex flex-col items-center">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary-50">
                <StarIcon className="h-6 w-6 text-primary-500" />
              </div>
              <h3 className="mt-4 text-sm font-semibold text-gray-900">Smart Validation</h3>
              <p className="mt-2 text-sm text-gray-600">Advanced response evaluation</p>
            </div>
            <div className="flex flex-col items-center">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary-50">
                <CommandLineIcon className="h-6 w-6 text-primary-500" />
              </div>
              <h3 className="mt-4 text-sm font-semibold text-gray-900">CLI First</h3>
              <p className="mt-2 text-sm text-gray-600">Built for developers</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Hero