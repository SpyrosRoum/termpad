# termpad

termpad allows you to easily host a pastebin server for saving and viewing text right from the terminal, or the browser.

## Client Usage
Assuming termpad is running in localhost:8000 you can do this to save text:  
```shell
$ curl -d "Hello world" localhost:8000
http://localhost/DullMagnificentLock
$ # Or
$ curl --data-binary @file localhost:8000
http://localhost/BrightAliveMotorcycle
```

And this to retrieve saved text:
```shell
$ curl localhost:8000/raw/TenderCheerfulYacht
$ # You might optionally want to pipe the output to a pager like less
$ curl localhost:8000/raw/TenderCheerfulYacht | less
```
Note the `/raw/` in the url, without it you will get html output


## Server Usage
### Note that environment variables are checked before assigning the default value.
### Domain (`-d` or `--domain`, env = `DOMAIN`)
Used to construct the url returned to the user. `http` is added as a prefix (Default: `localhost`)
```shell
$ termpad -d example.com
```
This will return urls like: `http://example.com/BrightAliveMotorcycle`

### Port (`-p` or `--port`)
Set the port on which the app runs (Default: `8000`)
```shell
$ termpad -p 8043
```

### Output (`-o` or `--output`)
Relative or absolute path to the directory where you want to store user-posted pastes (Default: `~/.local/share/termpad/`)
```shell
$ termpad -o /home/www/pastes/
```

### https (`--https`, env = `HTTPS`)
If set, urls will start with `https` instead of `http`
```shell
$ termpad --https
```

## Install
### From source:
```shell
$ git clone https://github.com/SpyrosRoum/termpad.git
$ cd termpad
$ cargo build --release
$ ./target/release/termpad
```

### With docker-compose:
Either `wget https://raw.githubusercontent.com/SpyrosRoum/termpad/master/docker-compose.yml` or copy the following into `docker-compose.yml`
```
version: "3.4"

services:
  app:
    image: spyrosr/termpad
    ports:
      - 8000:8000
    environment:
      - DOMAIN_NAME=example.com
      - HTTPS=true
    volumes:
      - data:$HOME/.local/share/termpad/
    restart: always

volumes:
  data:
    name: termpad
```
and then `docker-compose up -d`
