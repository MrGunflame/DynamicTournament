# DynamicTournament Web


## Building

```
make build
```

Include the following two `<link>` tags and the `<script>` tag into the page you want to embed the frontend. **Note that the `<link>` tags must be inside the `<head>` tag and `<script>` tag must load after the mountpoint.***

Adjust `/dynamic_tournament_web.js` and `/dynamic_tournament_web_bg.wasm` to the paths of the files.

```html
<head>
    <link rel="preload" href="/dynamic_tournament_web_bg.wasm" as="fetch" type="application/json">
    <link rel="modulepreload" href="/dynamic_tournament_web.js">
</head>
<body>
    <script type="module">
        import init, { main } from '/dynamic_tournament_web.js';

        const config = {
            api_base: "http://localhost:3030/api",
        };

        async function run() {
            await init();
            main(config);
        }

        run();
    </script>
</body>
```
