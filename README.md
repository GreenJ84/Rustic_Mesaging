<h1 align="center">Rustic Messaging System</h1>
<p align="center">
  Real-time messaging system built in Rust with a focus on authentication, secure communication, connection management, and low-latency.
</p>
<p align="center">
  <strong>Tech:</strong> Rust • Yew • WebSockets • JWT • Redis • Nginx • YugabyteDB
</p>

<h2>📌 Overview</h2>
<p>
  The app is split into a Yew frontend, a Rocket-based API, and a YugabyteDB-backed data layer. The current setup also includes helper crates for local startup and test startup, so the same services can be brought up in different ways without duplicating the core app logic.
</p>

<h3>🏗️ Architecture</h3>
<ul>
  <li>The Yew client talks to the API for normal requests and uses WebSockets for real-time message delivery</li>
  <li>Authentication is handled with JWTs during request and connection setup</li>
  <li>The server validates token claims, expiration, and signature before granting access</li>
  <li>Messages are routed through active connection handlers rather than polling loops</li>
  <li>Connection and session state is kept intentionally simple so it can later move toward shared infrastructure</li>
</ul>
<p>
  The system separates authentication, connection handling, message routing, and persistence so each concern can evolve independently.
</p>

<h3>🔐 Authentication Flow</h3>
<ul>
  <li>User authenticates and receives a JWT</li>
  <li>The token is attached to subsequent authenticated requests and WebSocket initialization</li>
  <li>The server checks expiration, signature, and claims before accepting the session</li>
  <li>Valid sessions are tied to the user identity that was encoded in the token</li>
  <li>Expired or invalid tokens are rejected early so unauthorized connections never reach the chat flow</li>
</ul>
<p>
  Token lifecycle handling includes expiration checks and session validation to ensure secure communication channels.
</p>

<h3>⚡ Real-Time Communication</h3>
<ul>
  <li>WebSocket-based persistent connections for low-latency delivery</li>
  <li>Connection lifecycle management for connect, message, and disconnect events</li>
  <li>Structured message handling so messages can be validated and routed consistently</li>
  <li>A client/server split that keeps the browser UI light and the server responsible for shared state</li>
</ul>
<p>
  The system prioritizes efficient message routing, connection stability, and a UI that can stay responsive while the server handles the real-time work.
</p>

<h3>📈 Performance Considerations</h3>
<ul>
  <li>Designed to minimize connection overhead and avoid unnecessary polling</li>
  <li>Built to handle multiple concurrent WebSocket sessions efficiently</li>
  <li>Planned Redis integration for caching, fan-out, and shared session state</li>
  <li>Separation of state makes it easier to scale the backend horizontally later</li>
</ul>

<h3>🧱 Scaling Strategy (Planned)</h3>
<ul>
  <li>Redis will be implemented for pub/sub messaging and shared session state across nodes (Required to support horizontal scaling of WebSocket connections)</li>
  <li>Support horizontal scaling of WebSocket servers when traffic grows</li>
  <li>Decouple the messaging layer for a more event-driven architecture later</li>
</ul>
<p>
  The current implementation has connection state currently maintained in-memory per node, which simplifies development but limits horizontal scalability, planned implementation of Redis for shared state infrastructure will address this limitation.
</p>

<h2>🐳 Dockerization</h2>
<p>
The repository is already structured for containerized development. The backend and frontend each have their own Dockerfile, and the root compose file ties them together with YugabyteDB so the production-like stack can be reproduced locally.
</p>
<ul>
  <li>The server image uses a multi-stage Rust build so the compiled API binary can be copied into a smaller runtime image.</li>
  <li>The runtime server container exposes port <code>8000</code> and listens on <code>0.0.0.0</code> so other containers can reach it.</li>
  <li>The client image builds the Yew app with Trunk and serves the generated static files through Nginx.</li>
  <li>The client container exposes port <code>80</code>, which becomes the browser entry point in Compose.</li>
  <li>Nginx proxies <code>/api</code> requests to the server container so the frontend and API behave like a single site from the browser's point of view.</li>
</ul>
<p>
This setup keeps build tooling inside the image, keeps runtime images focused on serving the app, and makes the same stack easier to run in a deployment pipeline or on another machine.
</p>

<h3>🧩 Compose Setup</h3>
<p>
The compose file wires the three services together: <code>yugabyte</code>, <code>server</code>, and <code>client</code>. Each service has a narrow responsibility so the stack is easy to reason about and the startup order stays predictable.
</p>
<ul>
  <li><code>yugabyte</code> provides the PostgreSQL-compatible database port used by Diesel and the Rust server.</li>
  <li><code>server</code> waits for YugabyteDB to become ready, resets the schema, runs migrations, and then launches the API.</li>
  <li><code>client</code> is built from the frontend Dockerfile and served by Nginx on host port <code>80</code>.</li>
  <li>The server attaches to both the frontend and backend networks so it can reach the database and accept requests from the Nginx container.</li>
  <li>The database uses a bind-mounted data directory so local state persists across container restarts.</li>
</ul>
<p>
  The compose layout is intentionally simple: one service for persistence, one for the API, and one for the browser-facing frontend. That makes startup order, port mapping, and debugging much easier while still matching the shape of the app in production.
</p>


<h2>▶️ Running the Project</h2>
<p>There are 2 ways to run the project:</p>
<ul>
  <li>*local environment*: starting each service individually for local development</li>
  <li>*Docker environment*: starting the full stack through Docker Compose in a production-like environment</li>
</ul>
<p>Each approach has its own use case and setup requirements.</p>

<h3>Repository Setup</h3>
<p>These setup steps are shared by both workflows.</p>
<ol>
  <li>Clone the repository.</li>
  <li>Open the workspace root so the helper crates can be run from a single terminal context.</li>
  <li>Install Docker (and Docker Compose if you want to run the compose stack).</li>
  <li>Create the local server environment settings from .env templates: <code>server/.env.template</code> and <code>server/.env.prod.template</code> and fill in according to template instructions.</li>
</ol>

```bash
git clone https://github.com/GreenJ84/Rustic_Mesaging.git #1
cd Rustic_Mesaging #2

curl  -fsSL https://get.docker.com  | sh #3

mv server/.env.template server/.env #4
mv server/.env.prod.template server/.env.prod #4
```


<h3>Local Environment</h3>
<p>This path uses a custom package to bring up the database, reset the schema, launch the API, and serve the frontend.</p>
<ol>
  <li>Install Rust and Cargo with rustup; if not already installed</li>
  <li>Install Trunk and the Diesel CLI; if not already installed</li>
  <li>Make sure Docker is running, because the startup package creates a YugabyteDB container before running tests.</li>
  <li>From the workspace root, start the local development helper package, <code>app_startup</code>.</li>
  <li>Wait for YugabyteDB to start, the schema to reset, and the API to launch.</li>
  <li>Open the API at <code>http://localhost:8000</code>.</li>
  <li>Open the frontend at <code>http://localhost:3000</code>.</li>
</ol>

```bash
rustup toolchain install stable #1
rustup default stable #1

cargo install trunk #2
cargo install diesel_cli --no-default-features --features postgres #2

docker info #3 - Should not have an error connecting to the Docker daemon

cargo run -p app_startup #4
```

<h3>Docker Compose Environment</h3>
<p>This path starts the database, API server, and Nginx-served frontend together as one stack.</p>
<ol>
  <li>From the repository root, build and start the compose stack.</li>
  <li>Wait for YugabyteDB, the API container, and the Nginx client container to start.</li>
  <li>Open <code>http://localhost</code> in your browser.</li>
  <li>Use <code>docker compose down</code> when you want to stop the stack.</li>
</ol>

```bash
docker compose up --build

docker compose down
```

<h2>🪪 License</h2>
<p>
  This project is licensed under the MIT License - see the <a href="/License.md">LICENSE.md</a> file for details.
</p>
<p>
  The MIT License is a permissive license that allows users to use, copy, modify, merge, publish, distribute, and sublicense the software, provided that they include the original copyright notice and disclaimer. It also provides an implied warranty of fitness for a particular purpose and limits the liability of the software's authors and contributors.
</p>
<p>
  By using or contributing to this project, you agree to be bound by the terms and conditions of the MIT License.
</p>
<p>
  If you have any questions about the license or would like to use this software under a different license, please contact the project maintainers.
</p>

<h2>🤗 Contributing</h2>
<p>Contributions are welcome!</p>
<p>
  Please refer to my profile <a href="https://github.com/GreenJ84/GreenJ84/blob/main/profile_code_of_conduct.md#contributor-code-of-conduct">Code of Conduct</a> before contributing to this project.
</p>
<p>
  My <a href="https://github.com/GreenJ84/GreenJ84/blob/main/profile_contributions.md.md#profile-contributions-guidline">Contribution Guide</a> has more details on how to get started contributing.
</p>
<p>
  Feel free to open an <a href="https://github.com/GreenJ84/Rustic_Mesaging/issues/new/choose">issue</a> or submit a <a href="https://github.com/GreenJ84/Rustic_Mesaging/compare">pull request</a> if you have a way to improve this project.
</p>
<p>
  Make sure your request is meaningful, thought out, and that you have tested the app locally before submitting a pull request.
</p>

<h2>🙋‍♂️ Support</h2>
<p>💙 If you like this project, give it a ⭐ and share it with friends!</p>
<p align="left">
  <a href="https://github.com/sponsors/GreenJ84">
    <img alt="Sponsor with Github" title="Sponsor with Github" src="https://img.shields.io/badge/-Sponsor-ea4aaa?style=for-the-badge&logo=github&logoColor=white"/>
  </a>
</p>

<!-- [☕ Buy me a coffee]() -->

<hr/>

<p>Made with Rust, Yew, Rocket, Redis, YugabyteDB, and ❤️‍🔥</p>
