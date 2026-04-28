# Analisis de Riesgo: Cofactor Clearing en `semaphore-rs`

## Pregunta

Evaluar si este proyecto tiene vulnerabilidades asociadas a **cofactor clearing**, usando como marco:

- `https://ristretto.group`
- el paper [Decaf_ Eliminating cofactors through point compression.pdf](/home/lorenzo/Desktop/Papers/Decaf_%20Eliminating%20cofactors%20through%20point%20compression.pdf)

## Resumen ejecutivo

Conclusion corta:

- **La parte principal de Semaphore proof generation no parece vulnerable hoy a ataques de cofactor clearing**, porque trabaja con secretos en el subgrupo primo, deriva la clave publica por multiplicacion fija del generador del subgrupo y el circuito tambien fuerza `secret < l`.
- **La API de firmas sobre Baby Jubjub si tiene un riesgo real/latente**, porque acepta `PublicKey` arbitrarias que solo se validan con `is_on_curve()`, no con chequeo de pertenencia al subgrupo primo.
- **El proyecto no implementa una abstraccion tipo Decaf/Ristretto**. Eso no rompe automaticamente a Semaphore, pero deja abierta la clase de bugs que Decaf/Ristretto justamente buscan eliminar si en el futuro se exponen puntos publicos serializados o se usa la API de firmas con input no confiable.

Mi evaluacion:

- **Proof system / identity commitments**: riesgo **bajo** por cofactor clearing.
- **Signature API (`identity.rs`)**: riesgo **medio-alto** si se usa con claves publicas provistas por terceros.

## Marco conceptual: que dicen Decaf y Ristretto

La idea central de ambos textos no es simplemente “multiplica por el cofactor y listo”.

Segun Ristretto, los problemas de una curva con cofactor aparecen porque la implementacion real entrega un grupo de orden compuesto `h * l`, mientras que muchos protocolos estan especificados y probados sobre un grupo de orden primo `l`. Eso genera:

- ataques de subgrupo pequeno,
- comportamiento no inyectivo,
- malleability por agregar torsion de bajo orden,
- diferencias de comportamiento entre implementaciones,
- y pruebas de seguridad que ya no aplican limpiamente.

El paper de Decaf es todavia mas explicito: para eliminar correctamente el cofactor no alcanza con “clear the cofactor” en algunos lugares. Hace falta una solucion integral sobre la abstraccion del grupo, incluyendo:

1. igualdad correcta entre representantes equivalentes,
2. encoding canonico,
3. decoding con validacion que rechace puntos fuera del conjunto permitido.

Ese es el punto clave para analizar este repo: **si el proyecto solo limpia el cofactor en algunos calculos, o si realmente construye una abstraccion de grupo primo**.

## Lo que hace hoy este proyecto

## 1. La curva usada tiene cofactor 8

En [src/baby_jubjub.rs](/home/lorenzo/Desktop/semaphore-rs/src/baby_jubjub.rs:35) el proyecto define Baby Jubjub con:

- cofactor `8`,
- subgrupo primo de orden `l`,
- y un `GENERATOR` que ya esta elegido dentro del subgrupo primo.

Eso es una buena base, pero no resuelve por si solo los problemas de cofactor si despues se aceptan puntos arbitrarios.

## 2. Las identidades honestas se generan dentro del subgrupo primo

En [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:121), `Identity::gen_secret_scalar()`:

- hace pruning del hash privado,
- fuerza divisibilidad por 8,
- luego divide por 8,
- y produce un escalar en `Fr`.

Despues, [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:157) usa:

```rust
let point = BabyJubjubConfig::GENERATOR.mul(secret_scalar).into_affine();
```

Como `GENERATOR` ya esta en el subgrupo primo, las claves publicas generadas por `Identity::new()` quedan en ese subgrupo. Esto reduce mucho el riesgo en el camino “honesto” del protocolo.

## 3. El circuito de Semaphore tambien fuerza el subgrupo correcto

En [circuits/src/semaphore.circom](/home/lorenzo/Desktop/semaphore-rs/circuits/src/semaphore.circom:37) el circuito exige:

```circom
secret < l
```

y calcula la clave publica con `BabyPbk()(secret)`.

O sea: la parte ZK no toma puntos arbitrarios desde afuera; toma un escalar privado acotado al orden primo y deriva la clave publica dentro del circuito. Eso es exactamente el tipo de flujo que evita muchos bugs de cofactor.

## 4. Los grupos de Semaphore almacenan commitments, no puntos

El Merkle tree en [src/group.rs](/home/lorenzo/Desktop/semaphore-rs/src/group.rs:55) no almacena puntos Edwards crudos. Almacena `Element = [u8; 32]`, y en la practica los miembros de Semaphore son commitments calculados como:

```rust
Poseidon(Ax, Ay)
```

desde [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:164).

Eso importa porque muchos problemas tipo Decaf/Ristretto aparecen cuando el protocolo trabaja directamente con puntos que pueden diferir por torsion. Aca, la capa de membresia trabaja con el hash de las coordenadas afines exactas, no con una clase cociente de puntos.

## Donde aparece el problema real

## La API publica acepta puntos “on curve” pero no valida subgrupo

La pieza mas delicada esta en [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:151):

```rust
pub fn from_point(point: EdwardsAffine) -> Self {
    Self { point }
}
```

No hay chequeo de subgrupo.

Y en [src/identity.rs](/home/lorenzo/Desktop/semaphore-rs/src/identity.rs:205), `Signature::verify()` solo valida:

- que `R` este en la curva,
- que `public_key` este en la curva.

Pero **no valida que `R` ni `public_key` esten en el subgrupo primo**.

Eso es exactamente el tipo de hueco que Ristretto/Decaf consideran peligroso: el sistema trabaja sobre una curva con cofactor, pero la API expone una abstraccion mas parecida a “cualquier punto de la curva” que a “elemento canonico de un grupo primo”.

## Por que multiplicar por el cofactor en verificacion no alcanza

La verificacion hace esto:

```rust
c_fr *= cofactor;
left = s * G;
right = R + c * A;
```

mas exactamente, despues del ajuste:

```text
sB = R + 8cA
```

Esto es una mitigacion parcial, no una validacion de subgrupo.

Ristretto dice explicitamente que **multiplicar por el cofactor “mangles the point” pero no valida pertenencia al subgrupo primo**. El paper de Decaf tambien advierte que reemplazar `P` por `P + T` con `T` de bajo orden sigue siendo una fuente de problemas si la abstraccion del grupo no esta bien cerrada.

## Consecuencia concreta: forja trivial para claves publicas de bajo orden

Si un atacante puede hacer que un verificador use una `PublicKey` de bajo orden `T`, entonces:

- `8cT = 0`,
- la ecuacion de verificacion queda reducida a:

```text
sB = R
```

que el atacante puede satisfacer eligiendo `R = sB`.

Caso todavia mas simple:

- usar la identidad `(0,1)` como clave publica,
- usar `R = (0,1)`,
- usar `s = 0`.

La identidad esta en la curva, asi que pasa `is_on_curve()`.
Como el termino con la clave publica desaparece al multiplicar por `8`, la firma deja de depender de la clave publica y del mensaje de una manera criptograficamente sana.

En otras palabras:

- **si la API de firmas se usa con claves publicas externas no confiables, hay riesgo de forgery/existencia de firmas validas bajo claves de bajo orden**;
- el chequeo actual no lo evita.

## Importante: esto afecta a Semaphore como protocolo principal?

**No de forma directa, en lo que hoy veo en el repo.**

La razon:

- `Proof::generate_proof()` toma una `Identity`, no una clave publica arbitraria.
- La `Identity` honesta siempre se deriva dentro del subgrupo primo.
- El circuito usa `secret < l` y deriva el commitment desde el escalar.
- El Merkle tree almacena commitments, no puntos crudos.

Por eso, **el flujo principal de Semaphore no parece depender de una “limpieza incompleta” del cofactor** del modo en que fallan protocolos que aceptan puntos arbitrarios externos.

## Entonces, cual es el diagnostico correcto?

## No hay una vulnerabilidad estructural obvia en el flujo ZK principal

No encontre evidencia de que un atacante pueda:

- inyectar un punto con torsion en el circuito,
- producir dos representantes torsion-equivalentes de una misma identidad aceptados por el mismo commitment,
- ni romper la logica de nullifier o de Merkle membership por falta de una abstraccion tipo Ristretto.

El diseño actual evita bastante bien ese problema porque el “objeto criptografico publico” del protocolo no es un punto serializado, sino un `identity commitment` derivado de una clave generada honestamente dentro del subgrupo.

## Pero si hay una vulnerabilidad o debt criptografico en la capa de firmas

La API de firmas si cae en la clase de problemas que describen Decaf/Ristretto:

- curva con cofactor 8,
- puntos externos aceptados sin chequeo de subgrupo,
- cofactor clearing usado como compensacion algebraica,
- ausencia de encoding/decoding canonico de grupo primo.

Eso no significa que haya que reescribir todo el proyecto con Ristretto mañana.
Si significa que **la API actual no deberia tratarse como una abstraccion segura de “grupo primo sobre Baby Jubjub”**.

## Recomendaciones

## 1. Arreglo urgente si la API de firmas va a usarse externamente

Agregar validacion de subgrupo para:

- `PublicKey::from_point`
- `Signature::verify` sobre `public_key`
- idealmente tambien sobre `R`

La validacion correcta es del estilo:

- verificar que el punto este en la curva,
- verificar que `[l]P = O`,
- y segun el caso, rechazar tambien la identidad como clave publica valida.

Solo `is_on_curve()` no alcanza.

## 2. Aclarar el contrato de la API

Si quieren mantener `from_point`, documentar explicitamente:

- que espera un punto del subgrupo primo,
- que pasar puntos arbitrarios es inseguro,
- y que la funcion no hace validacion suficiente hoy.

Idealmente, cambiarla para que devuelva `Result<PublicKey, SemaphoreError>`.

## 3. No confiar en “cofactor clearing” como sustituto de subgroup membership

La leccion mas importante de Ristretto/Decaf para este repo es:

- **clearing no es validation**.

Si el protocolo necesita aceptar puntos externos, hace falta una abstraccion mas fuerte.

## 4. Evaluar si realmente hace falta una capa tipo Decaf/Ristretto

Mi recomendacion pragmatica:

- **para el flujo principal de Semaphore, probablemente no hace falta migrar todo a una abstraccion tipo Ristretto**;
- **para la API de firmas o cualquier futura serializacion de puntos Baby Jubjub, si conviene al menos imponer subgroup membership y encoding canonico**.

Implementar un “Ristretto para Baby Jubjub” no es un parche chico. Solo vale la pena si el repo quiere exponer puntos publicos como objetos interoperables y aceptarlos desde input no confiable.

## Nivel de severidad

## Flujo principal de Semaphore

- Severidad: **baja**
- Estado: **sin hallazgo explotable claro por cofactor clearing**

## API de firmas en `identity.rs`

- Severidad: **media-alta**
- Estado: **hallazgo real/latente**
- Condicion de explotacion: que un consumidor del crate acepte claves publicas Baby Jubjub no confiables usando `PublicKey::from_point`

## Conclusiones finales

Este proyecto **no implementa Decaf/Ristretto**, pero tampoco parece necesitarlo para el corazon del protocolo Semaphore tal como esta hoy, porque su flujo principal evita puntos externos y trabaja con secretos/commitments dentro del subgrupo correcto.

El problema aparece en la frontera donde el crate expone una API de firmas sobre una curva con cofactor 8 sin hacer validacion de subgrupo. Ahi si hay una deuda criptografica concreta, y el mensaje de Ristretto/Decaf aplica de lleno: **no alcanza con limpiar el cofactor en una ecuacion; si aceptas puntos externos, necesitas una abstraccion de grupo primo o al menos una validacion estricta del subgrupo**.

## Referencias

- Ristretto home: <https://ristretto.group/>
- Why Ristretto?: <https://ristretto.group/why_ristretto.html>
- What is Ristretto?: <https://ristretto.group/what_is_ristretto.html>
- Ristretto in Detail: <https://ristretto.group/details/index.html>
- Paper: [Decaf_ Eliminating cofactors through point compression.pdf](/home/lorenzo/Desktop/Papers/Decaf_%20Eliminating%20cofactors%20through%20point%20compression.pdf)
