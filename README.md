# Azusa

The File Downloader with REST API

## Develop

### Docker
Start server:

```shell
git clone git@github.com:techno-tanoC/azusa.git
cd azusa
docker-compose up
```

Start download:

```shell
curl -H 'Content-Type:application/json' -d '{ "url": "https://releases.ubuntu.com/20.04/ubuntu-20.04-desktop-amd64.iso", "name": "test", "ext": "iso" }' http://localhost:3000/download
open http://localhost:3000/assets/index.html
```

### Rust + nodejs
Start server:

```shell
git clone git@github.com:techno-tanoC/azusa.git
cd azusa
yarn install
yarn build
cargo run --release
```

Start download:

```shell
curl -H 'Content-Type:application/json' -d '{ "url": "https://releases.ubuntu.com/20.04/ubuntu-20.04-desktop-amd64.iso", "name": "test", "ext": "iso" }' http://localhost:3000/download
open http://localhost:3000/assets/index.html
```
