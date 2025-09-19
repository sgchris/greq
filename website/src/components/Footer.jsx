import { 
  GitHubLogoIcon, 
  TwitterLogoIcon, 
  DiscordLogoIcon 
} from '@radix-ui/react-icons'

const Footer = () => {
  const navigation = {
    main: [
      { name: 'Documentation', href: 'https://github.com/sgchris/greq/blob/main/docs/documentation.md' },
      { name: 'Examples', href: 'https://github.com/sgchris/greq/tree/main/greq-examples' },
      { name: 'GitHub', href: 'https://github.com/sgchris/greq' },
      { name: 'Issues', href: 'https://github.com/sgchris/greq/issues' }
    ],
    social: [
      {
        name: 'GitHub',
        href: 'https://github.com/sgchris/greq',
        icon: GitHubLogoIcon
      },
      {
        name: 'Twitter',
        href: 'https://twitter.com',
        icon: TwitterLogoIcon
      },
      {
        name: 'Discord',
        href: 'https://discord.com',
        icon: DiscordLogoIcon
      }
    ]
  }

  return (
    <footer className="bg-white border-t border-gray-200">
      <div className="mx-auto max-w-7xl px-6 py-12 md:flex md:items-center md:justify-between lg:px-8">
        <div className="flex justify-center space-x-6 md:order-2">
          {navigation.social.map((item) => (
            <a
              key={item.name}
              href={item.href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-400 hover:text-gray-500 transition-colors"
            >
              <span className="sr-only">{item.name}</span>
              <item.icon className="h-6 w-6" />
            </a>
          ))}
        </div>
        <div className="mt-8 md:order-1 md:mt-0">
          <div className="flex justify-center space-x-6 md:justify-start">
            {navigation.main.map((item) => (
              <a
                key={item.name}
                href={item.href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm leading-6 text-gray-600 hover:text-gray-900 transition-colors"
              >
                {item.name}
              </a>
            ))}
          </div>
          <p className="mt-4 text-center text-xs leading-5 text-gray-500 md:text-left">
            &copy; {new Date().getFullYear()} Greq. Built with ❤️ for developers.
          </p>
        </div>
      </div>
    </footer>
  )
}

export default Footer