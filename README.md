# we have ascii at home

Crappy selfhosted hardcodet static cheap clone of asciinema.org

Yoinked the player code straight from [release 3.2.0](https://github.com/asciinema/asciinema-player/releases/tag/v3.2.0)

https://ascii.zillyhuhn.com/?a=twnet

## Setup simple

Serve the ``frontend/`` folder with your favorite web server.

Then just put the .cast files your recorded with [asciinema](https://github.com/asciinema/asciinema) into the `frontend/casts/` folder.
It should just work with statically serving those files. No backend needed.

## Setup advanced

You can additionally opt in to spin up some backend services. These unlock views and comments.

```
TODO: document backend setup when backend is finished
```