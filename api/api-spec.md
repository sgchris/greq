# API service

The API service meant to serve requests coming from the static website under "/website". Currently there's on "subscribe" request but will be more in the future

All the service' files are located under "/api" folder.

# API description

## Endpoints

- /api/subscribe - An endpoint to which the "subscribe" requests will be sent.
- /api/get_subscribers - Displays list of subscribed users (emails).

## Technologies

- The API service will be written in PHP
- Use Slim framework
- Use SQLite database
- Use Composer. (it's already installed)
- The service must be stateless


## Notes

### Command line tools

- You may use any command line tool in order to build, maintain, test or any other require operation for the service development and maintenance
- The CLI commands must not require user input. All the information must be in the command itself.
- For CLI commands that might stuck or go out of the control, add an execution timeout

### Security

- Add CORS headers to API endpoints that will be used as AJAX requests. Allow requests from the same domain/origin.
- Endpoints that provide an HTML page (like /api/get_subscribers) are behind a password that will be statically defined in the config file
- Don't allow embedding
- Implement rate limiter. Be strict.
- Add protection against spamming and XSS.
- Allow requests from browsers only. No bots allowed.

### The frontend

The frontend of the API service is used to display information, like the list of subscribers, in a very simple way

- No JS frameworks or any build systems.
- Icons, fonts, CSS frameworks allowed (like tailwind, heroicons, etc.). Load them directly into the website using a CDN.
- Vanilla javascript only

