# minipak
An executable packer. Given an ELF executable as input, it outputs a compressed executable which you can then run.
Written while reading [this series](https://fasterthanli.me/series/making-our-own-executable-packer).
## Usage
`minipak input -o output`  

Example:  
```
# compress git
minipak /usr/bin/git -o /tmp/git.pak
# run the compressed git executable
/tmp/git.pak --version
```
