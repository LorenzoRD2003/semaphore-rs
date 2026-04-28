# AGENTS.md

## Resumen

Este repo implementa partes del protocolo **Semaphore** en tres frentes:

1. Una libreria Rust publicada como crate `semaphore-protocol`, cuyo nombre de libreria es `semaphore`.
2. Un paquete `circom` con el circuito `Semaphore` y tests en TypeScript.
3. Un subproyecto `semaphore-cardano/aiken` que explora una variante on-chain para Cardano.

La libreria Rust es el centro del repo. Los circuitos y el trabajo de Cardano son piezas relacionadas, pero no forman un solo build unificado.

## Estructura real del repo

- `src/`
  Codigo Rust principal.
- `tests/`
  Tests de integracion Rust para identidad y grupos.
- `witness_graph/`
  Binarios precomputados `semaphore-{1..32}.bin` usados para generar witnesses localmente segun la profundidad del arbol.
- `circuits/`
  Circuito `circom`, config de `circomkit` y tests de Node/TypeScript.
- `script/build_witness_graph.sh`
  Script para regenerar `witness_graph/` clonando repos externos.
- `semaphore-cardano/aiken/`
  Implementacion Aiken con validators, logica de nullifiers y tests.
- `.github/workflows/test.yml`
  CI principal del repo.

## Modulos Rust importantes

- `src/lib.rs`
  Expone los modulos publicos y fija `MIN_TREE_DEPTH = 1`, `MAX_TREE_DEPTH = 32`.
- `src/identity.rs`
  Modela `Identity`, `PublicKey` y `Signature`.
  Usa Baby Jubjub sobre BN254, Blake para derivacion y Poseidon para commitments.
- `src/group.rs`
  Wrapper sobre `zk-kit-lean-imt` con hash Poseidon.
  Maneja miembros, roots y merkle proofs.
- `src/proof.rs`
  Arma y verifica pruebas Groth16 de Semaphore.
  Une identidad + grupo + witness graph + `.zkey`.
- `src/witness.rs`
  Hace dispatch al witness generator correcto segun profundidad, usando los binarios de `witness_graph/`.
- `src/utils.rs`
  Helpers de conversion y descarga de `.zkey`.
- `src/error.rs`
  Errores del dominio.

## Flujo de prueba Rust

El pipeline real para `Proof::generate_proof` es:

1. Derivar `secret_scalar` y commitment desde `Identity`.
2. Obtener o construir una `MerkleProof` del `Group`.
3. Normalizar siblings hasta la profundidad pedida.
4. Hashear `scope` y `message` con `keccak256` truncado via `utils::hash`.
5. Descargar el `.zkey` correspondiente a la profundidad.
6. Ejecutar el witness generator embebido desde `witness_graph/semaphore-{depth}.bin`.
7. Generar la prueba Groth16 con `circom-prover`.

Para verificar:

1. Se rehace el hash de `scope` y `message`.
2. Se reconstruyen los public inputs.
3. Se vuelve a usar el `.zkey` de esa profundidad.

## Artefactos y dependencias operativas

### Rust

- Requiere `protobuf-compiler` (`protoc`) para compilar la dependencia `circom-witnesscalc`.
- CI instala `protobuf-compiler` antes de correr lint y tests.
- `cargo test --all-features` es el comando relevante de CI.

### Witness graphs

- `witness_graph/*.bin` es material esencial del repo, no basura generada.
- `src/witness.rs` depende de que existan archivos para todas las profundidades `1..=32`.
- Si se regeneran, usar `script/build_witness_graph.sh`.

### ZKey

- `src/utils.rs` descarga `.zkey` desde `https://snark-artifacts.pse.dev/semaphore/4.13.0/`.
- Los archivos se cachean en `std::env::temp_dir()` con nombre `semaphore-4.13.0-{depth}.zkey`.
- La generacion y verificacion de proofs depende de red si ese `.zkey` no existe ya en cache local.

### Circuits

- `circuits/package.json` usa `circomkit` y tests con `mocha`.
- Antes de correr tests ahi, hay que instalar dependencias de Node.
- En este workspace, `npm test` fallo porque `mocha` no estaba instalado, lo que indica que faltaba bootstrap de dependencias.

### Cardano / Aiken

- `semaphore-cardano/aiken` se construye con `aiken build`.
- `aiken check` corre 76 tests actualmente.
- La verificacion de proof en Aiken esta mockeada en v1: `verify_proof(pi)` acepta cualquier `pi != #""`.

## Comandos utiles

### Rust

```bash
cargo fmt --all
cargo clippy -- -D warnings
cargo test --all-features
```

Si `cargo test --all-features` falla por build de `circom-witnesscalc`, revisar primero que `protoc` exista en PATH.

### Circuits

```bash
cd circuits
npm install
npm test
```

Tambien existen scripts:

```bash
npm run compile
npm run setup
```

### Aiken

```bash
cd semaphore-cardano/aiken
aiken build
aiken check
```

### Regenerar witness graphs

```bash
./script/build_witness_graph.sh
```

Ese script:

- clona `iden3/circom-witnesscalc` si no existe localmente,
- clona `semaphore-protocol/semaphore` si no existe localmente,
- hace `yarn install` en el repo clonado,
- genera circuitos `semaphore-{1..32}.circom`,
- y recompila `witness_graph/semaphore-{1..32}.bin`.

## Estado verificado en este workspace

Al momento de escribir este archivo:

- `aiken check` paso con `76/76` tests.
- `cargo test --all-features` no llego a correr porque faltaba `protoc`.
- `cd circuits && npm test` fallo porque `mocha` no estaba disponible, o sea faltaban deps de Node.

## Gotchas importantes

### Nombres inconsistentes

- El paquete en `Cargo.toml` se llama `semaphore-protocol`.
- La libreria expuesta se llama `semaphore`.
- El `README.md` mezcla ejemplos con nombres como `semaphore` y `semaphore-rs`.

### Network dependency silenciosa

- Generar o verificar proofs puede disparar descarga de `.zkey`.
- Si estas trabajando offline, esa parte puede fallar aunque el codigo compile.

### Template drift en docs

- El PR template menciona `yarn style`, pero este repo principal no tiene ese script en raiz.
- Parte del texto de `README` y `CONTRIBUTING` parece heredado de otros repos de Semaphore.
- Tratar `Cargo.toml`, `src/`, `tests/` y `.github/workflows/test.yml` como fuente de verdad operativa.

### Profundidad del arbol

- Rust soporta explicitamente profundidades `1..=32`.
- `dispatch_witness` hace `panic!` fuera de ese rango.
- `Proof::generate_proof` y `Proof::verify_proof` asumen esa restriccion.

### Mensajes y scopes

- `Proof::generate_proof` recibe `message` y `scope` como `String`.
- `utils::to_big_uint` no interpreta decimal arbitrario: toma los bytes ASCII del string, los copia en 32 bytes big-endian y crea un `BigUint`.
- Eso significa que `"123"` no se trata como numero 123, sino como bytes de texto.

## Que tocar segun la tarea

- Cambios de identidad, firmas o commitments:
  mirar `src/identity.rs`.
- Cambios de Merkle tree, miembros o proofs de pertenencia:
  mirar `src/group.rs` y `tests/group.rs`.
- Cambios de generacion/verificacion de proofs:
  mirar `src/proof.rs`, `src/utils.rs`, `src/witness.rs`.
- Cambios del circuito:
  mirar `circuits/src/semaphore.circom` y `circuits/tests/`.
- Cambios del experimento Cardano:
  mirar `semaphore-cardano/aiken/lib/semaphore/` y `validators/`.

## Recomendaciones para futuros agentes

1. Antes de editar logica de proofs, confirmar si la tarea afecta Rust, `circom`, Aiken o mas de una capa.
2. No borrar ni regenerar `witness_graph/` salvo que la tarea lo pida explicitamente.
3. Si falla `cargo test`, revisar primero tooling del sistema (`protoc`) antes de asumir bug del crate.
4. Si la tarea toca `Proof`, documentar cualquier cambio de formato de public inputs o packing Groth16.
5. Si la tarea toca Cardano, recordar que hoy la verificacion criptografica on-chain aun no es real, solo mock.

