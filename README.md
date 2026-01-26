# Beets edit thingy


## How to use

This needs to be in the beets config:

```
# Extra fields to edit
edit:
  itemfields: 'track title album artist albumartist artists albumartists'
  albumfields: 'album albumartist albumartists genre genres'
```

Edit albums:

```
EDITOR="beets-edit edit-albums" beet edit -a $YOUR_QUERY
```

Edit tracks:

```
EDITOR="beets-edit edit-tracks" beet edit -a $YOUR_QUERY
```
