# [IST-SCOM: Multiparty Battleship Game](docs/Project-Description.pdf)
<img width="1065" height="664" alt="image" src="https://github.com/user-attachments/assets/80e66256-7fa8-46c2-a1af-6c0e8008ad49" />

## Project Overview
This project implements a privacy-preserving, multiplayer variant of the Battleship game using zkSNARKs (Zero-Knowledge Succinct Non-interactive Arguments of Knowledge) to enforce the rules without relying on a third party, built on the RISC Zero architecture, a zero-knowledge verifiable general computing platform. **The goal is to ensure that all players follow the game’s logic**, such as placing valid ship configurations, reporting shot outcomes truthfully, and winning only when conditions are satisfied, **while keeping sensitive information** (i.e., fleet configuration) **private throughout the game**. 

A more thorough description of the project aswell as the implementation can be found in the [docs directory](docs).

## **Directory Structure**
This project follows a modular structure inspired by the [RISC Zero Rust Starter Template](https://github.com/risc0/risc0), but separates each logical component into its own crate under `src/`. Each subdirectory (e.g. `host`, `blockchain`, `fleetcore`, `methods/guest`) contains its own `Cargo.toml` and `src/`.

```tree
src/
├── blockchain/
├── fleetcore/
├── host/
└── methods/
    ├── guest/
    └── src/
```

In a RISC Zero project, the terms **methods** and **guest** are part of the framework's architecture that enables **zero-knowledge proofs of computation**.


## **How to Run the Project with Docker**
###  1. **Build and start all containers**

Run this once to build and launch the containers in the background:

```bash
docker-compose up --build -d
```
Note that:
- `--build` forces a rebuild of the Docker image using the Dockerfile (needed only if an alteration to said file was made).
- `-d` starts the containers in the background. If this flag is not used the terminal will display:
  ```bash
  [+] Running 2/2
  ✔ Container chain0   Created                    0.0s
  ✔ Container player0  Created                    0.0s
  Attaching to chain0, player0
  ```
This:
* Builds the Docker image from the `Dockerfile`
* Starts all services defined in `docker-compose.yml`  (`player0`, `chain0`)
* Mounts the source code directory to `/workspace`

### 2. **Enter a container terminal**
To enter the shell of a specific container:
```bash
# Enter the player container
docker exec -it player0 bash

# Enter the chain container
docker exec -it chain0 bash
```

### 3. **Run the program instances in each container**

Inside each container, use Cargo to run each program instance:
```bash
# For a player:
cd src/host/src
cargo run --bin host

# For the chain:
cd src/blockchain/src
cargo run --bin blockchain
```

### 4. **Stop the containers**

To stop the containers, either write:
```bash
docker-compose down
```
Or if the `-d` flag was not used, simply use `Ctrl + c` on the terminal `docker-compose up` was invoked
