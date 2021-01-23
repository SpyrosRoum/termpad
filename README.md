# termpad

termpad allows you to easily host a TCP server for saving and viewing text right from the terminal.

## Client Usage
You will need netcat if you want to use termpad from the terminal.  
By default there will be no website where you can see the text you save:

Assuming you have netcat installed and a termpad server running in localhost you can do this to save text:  
```shell
$ echo "Hello World!" | nc localhost 9999
AngryDog
$ # Or
$ cat file | nc localhost 9999
ModernAverageOcean
```

And this to retrieve saved text:
```shell
$ echo "ModernAverageOcean" | nc localhost 8888
$ # You might optionally want to pipe the output to a pager like less
$ echo "AngryDog" | nc localhost 8888 | less
```

If the termpad server running has a webserver too, urls will be returned instead of just keys (`https://localhost/ModernAverageOcean`).  
In this case you can either open the url or use the key in the same way as before.


## Server Usage
If termpad has been compiled **without** the `web` flag these are the options you have when running it:

### Buffer Size (`-B` or `--buffer-size`)
This parameter defines size of the buffer used for getting data from the user.   
Maximum size (in bytes) of all input files is defined by this value. (Default: 50.000)  
```shell
$ termpad -B 1024
```

### Output (`-o` or `--output`)
Relative or absolute path to the directory where you want to store user-posted pastes (Default: `./pastes/`)
```shell
$ termpad -o /home/www/pastes/
```
 
If termpad was compiled **with** the `web` flag you have a few more options:

### Domain (`-d` or `--domain`)
Used to construct the url returned to the user. `http` is added as a prefix (Default: `localhost`)
```shell
$ termpad -d example.com
```
This will return urls like: `http://example.com/AngryDog`

### https (`--https`)
If set, urls will start with `https` instead of `http`
```shell
$ termpad --https
```

## Install
### From source:
```shell
$ git clone https://github.com/SpyrosRoum/termpad.git
$ cd termpad
$ # If you want to compile with web server support:
$ cargo build --release --features web
$ # If you only want to use it from the terminal:
$ cargo build --release
$ ./target/release/termpad
```
