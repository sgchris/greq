# Software design

## Params
- Create object with all the default values. Call it Opts
- Parse the parameters and override the default Opts
- Opts should be defined with Arc in 'main' (several threads will use it)

## Definitions
- headers - Parse into a dictionary
- The request - Take as is 
- Footer - Parse into a list of conditions
+ every condition is a key, value, optional not, optional or
+ Not or Or may be defined as enum. The condition prefix might be as list of these enums
- greq - the whole request object containing the three parts - header, content, footer


## Single thread
- pass filename and Opts
- parse the content - divide into 3 parts and create object from every part
- Create greq object
- build the request object
- execute the request
- check if the response has to be saved
- run conditions (if there are)
- return result


## Things to remember
- Handle cancel interrupt (Ctrl-C)



