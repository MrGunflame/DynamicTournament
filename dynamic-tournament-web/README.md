# DynamicTournament Web


## Building

```
make build
```

Include the following two `<link>` tags and the `<script>` tag into the page you want to embed the frontend. **Note that the `<link>` tags must be inside the `<head>` tag and `<script>` tag must load after the mountpoint.***

Adjust `/dynamic_tournament_web.js` and `/dynamic_tournament_web_bg.wasm` to the paths of the files. Include the `./dynamic_tournament_web_index.css` file. If you're not embedding also include `./dynamic_tournament_web_hosted.css`.

Include the entire `assets` directory.

```html
<head>
    <link rel="preload" href="/dynamic_tournament_web_bg.wasm" as="fetch" type="application/wasm">
    <link rel="modulepreload" href="/dynamic_tournament_web.js">

    <link rel="stylesheet" href="./dynamic_tournament_web_index.css">
    <link rel="stylesheet" href="./dynamic_tournament_web_hosted.css">
    <link rel="stylesheet" href="./assets/fontawesome/css/all.min.css">
</head>
<body>
    <script type="module">
        import init, { main } from '/dynamic_tournament_web.js';

        const config = {
            api_base: "http://localhost:3030/api",
            root: "/",
        };

        async function run() {
            await init();
            main(config);
        }

        run();
    </script>
</body>
```
