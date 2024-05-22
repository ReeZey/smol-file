# smol - 5 bit encoding file format
what is smol? smol is a file format that compresses text into 5 bits per letter instead of the normal 8 bits.
this is achived by having a charset lesser than 32 chars so all letters fits nicely into 5 bits (32 values). 
this makes all sort of problem such as an byte being 8 so multiple letters will overlap eachother
but this is all handeled through this library for ease of use.  
  
this is not made for any production applications only as an hobby.