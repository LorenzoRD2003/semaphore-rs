# Decaf/Ristretto y la Migración de Semaphore a Cardano

Este documento resume cómo impacta el marco conceptual de Decaf/Ristretto sobre el diseño descripto en [semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex).

## Conclusión corta

La migración de Semaphore a Cardano sobre BLS12-381, Jubjub o Bandersnatch **no obliga por sí sola** a usar Decaf o Ristretto.

Pero sí hace visible una distinción importante:

- **ajustar el pruning y trabajar con un generador del subgrupo primo** ayuda a producir identidades honestas;
- **eso no reemplaza** una abstracción de grupo primo al estilo Decaf/Ristretto si el sistema va a aceptar, serializar o manipular puntos públicos externos.

En otras palabras:

- para el core del proof system, Decaf/Ristretto no parece ser la pieza central;
- para APIs, enrolamiento, interoperabilidad o validación de claves públicas fuera del circuito, sí se vuelve altamente relevante.

## Qué dice hoy `semaphore-cardano.tex`

El documento modela la identidad como:

1. un escalar secreto derivado por hash y pruning,
2. una clave pública sobre una curva Edwards,
3. un `identity commitment = Poseidon(Ax, Ay)`.

Referencias:

- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:45)
- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:69)
- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:281)

También afirma correctamente que al migrar de BN254/Baby Jubjub a BLS12-381/Jubjub/Bandersnatch hay que cambiar:

- la curva embebida en el circuito,
- el generador efectivo del subgrupo primo,
- la derivación del secreto,
- y parámetros del hash.

Referencia:

- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:386)

## Dónde Decaf/Ristretto sí importa

Decaf/Ristretto no se limita a “limpiar el cofactor”. Su mensaje principal es que, en curvas con cofactor, una aplicación segura suele necesitar además:

- encoding canónico,
- decoding restringido,
- igualdad compatible con la torsión,
- y validación explícita de subgroup membership.

Eso se vuelve importante si la migración a Cardano termina incluyendo cualquiera de estos casos:

- claves públicas de usuario serializadas fuera del circuito,
- enrolamiento basado en “prueba de posesión” de una clave,
- interoperabilidad entre implementaciones distintas,
- APIs que aceptan puntos Edwards desde input no confiable,
- o cualquier capa off-chain que razone sobre puntos de curva en lugar de commitments.

En ese escenario, pruning y cofactor clearing ya no alcanzan como historia de seguridad completa.

## Dónde importa poco

Para el flujo central de Semaphore, bastante menos.

Según el documento, el circuito:

- recibe un escalar secreto,
- reconstruye la clave pública dentro del circuito,
- hashea esa clave a un commitment,
- y prueba pertenencia del commitment al árbol.

Referencias:

- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:274)
- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:303)

Ese diseño evita gran parte de la superficie clásica de Decaf/Ristretto, porque el verificador no recibe puntos Edwards arbitrarios como inputs principales del protocolo. Recibe una prueba Groth16 y valores públicos como `merkleRoot`, `nullifier`, `message` y `scope`.

Por eso, en el core del protocolo, los riesgos más urgentes no parecen ser de la clase Decaf/Ristretto sino otros más directos, como:

- aceptar identidades degeneradas como `secret = 0`,
- validar correctamente subgroup membership en cualquier capa off-chain,
- y tener una verificación real de Groth16.

## Dónde hoy no importa casi nada

En la implementación Aiken actual, el impacto práctico de Decaf/Ristretto es todavía menor porque la verificación criptográfica on-chain todavía no está implementada de verdad.

Hoy:

- `verify_proof(pi)` es un mock,
- y acepta cualquier `pi != #""`.

Referencia:

- [semaphore-cardano/aiken/lib/semaphore/proof_logic.ak](/home/lorenzo/Desktop/semaphore-rs/semaphore-cardano/aiken/lib/semaphore/proof_logic.ak:5)

Mientras eso siga así, el principal cuello de botella de seguridad no es el cofactor ni la abstracción de grupo, sino la ausencia de verificación zk real on-chain.

## Qué cambiaría en la lectura del documento

La sección sobre cofactor y pruning está bien orientada, pero conviene no presentar esa parte como suficiente por sí sola.

Referencia:

- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex:127)

La versión más precisa sería:

- pruning y elección del generador correcto ayudan a que las identidades honestas vivan en el subgrupo primo;
- pero no sustituyen una abstracción tipo Decaf/Ristretto si se aceptan puntos externos;
- y tampoco sustituyen validaciones explícitas contra identidades degeneradas.

## Recomendación de diseño

Para esta migración, la postura más razonable parece ser:

- no hacer de Decaf/Ristretto un requisito obligatorio del protocolo base;
- sí tratar sus lecciones como guía de diseño para cualquier interfaz off-chain que toque puntos de curva;
- y dejar explícito en la documentación que:
  - subgroup membership,
  - canonical encoding,
  - rechazo del punto identidad,
  - y validación de inputs externos
  son preocupaciones separadas del pruning.

## Conclusión final

Decaf/Ristretto **no parece ser la pieza central** para que Semaphore funcione sobre Cardano si el diseño sigue centrado en:

- secretos dentro del circuito,
- commitments en el árbol,
- y verificación zk como frontera criptográfica principal.

Pero sí es un marco muy útil para evitar errores de implementación alrededor de la identidad si la migración agrega capas donde se manipulen puntos públicos de curva fuera del circuito.

La frase correcta no es:

`"con cambiar el cofactor y el pruning alcanza"`

sino más bien:

`"eso alcanza para producir identidades honestas, pero no para reemplazar una abstracción de grupo primo cuando hay puntos públicos externos en juego"`

## Referencias

- [docs/semaphore-cardano.tex](/home/lorenzo/Desktop/semaphore-rs/docs/semaphore-cardano.tex)
- [Aiken README](/home/lorenzo/Desktop/semaphore-rs/semaphore-cardano/aiken/README.md)
- [proof_logic.ak](/home/lorenzo/Desktop/semaphore-rs/semaphore-cardano/aiken/lib/semaphore/proof_logic.ak)
- Ristretto: <https://ristretto.group/>
- Why Ristretto?: <https://ristretto.group/why_ristretto.html>
- Decaf paper: [Decaf_ Eliminating cofactors through point compression.pdf](/home/lorenzo/Desktop/Papers/Decaf_%20Eliminating%20cofactors%20through%20point%20compression.pdf)
- ZIP 216: <https://zips.z.cash/zip-0216>
- Bandersnatch paper: <https://eprint.iacr.org/2021/1152>
