# arquita

CLI en Rust para emitir **facturas electrónicas** de Argentina contra los web services de **ARCA/AFIP**, hablando directamente con la API de AFIP — sin depender de ningún proxy en la nube ni access token de terceros.

## Requisitos

- Rust 1.85+ (edición 2024).
- CUIT con **Clave Fiscal nivel 3** (o superior).
- Estar inscripto en un régimen que emita **Factura C** (p. ej. **Monotributo**), con actividad registrada.
- Un **punto de venta habilitado para Web Services** (ver [Setup](#1-crear-el-punto-de-venta-en-arca)).

Cada entorno (homologación y producción) usa **certificados y trámites distintos**: un certificado de testing no sirve en producción y viceversa.

## Build

```bash
cargo build --release
# binario en target/release/arquita
```

## Setup

Pasos que se hacen **una sola vez** antes de poder emitir facturas. Todos los comandos trabajan sobre un directorio de trabajo (`--home`, por defecto `$AFIP_HOME` o `~/arquita`) que contiene la configuración, los certificados y la caché de credenciales.

### 1. Crear el punto de venta en ARCA

Necesitás un punto de venta **habilitado para Web Services**, que es **distinto** del que se usa usalmente en «Comprobantes en Línea»:

1. En el portal de ARCA entrá al servicio **«Administración de puntos de venta y domicilios»**.
2. Elegí **«A/B/M de puntos de venta»** y agregá uno nuevo.
3. En **Sistema** seleccioná según tu condición: **Monotributo** → «Factura Electrónica – Monotributo – Web Service»; **Responsable Inscripto** → «RECE para aplicativo y Web Service».
4. Confirmá con **«Aceptar»**.

Anotá el **número de punto de venta**: es el que va en `--punto-venta`. 

Para una guía detallada: [Crear punto de venta (Afip SDK)](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/crear-punto-de-venta).

### 2. Configurar el emisor

```bash
arquita init \
  --cuit 20111111112 \
  --punto-venta 1 \
  --razon-social "Mi Nombre" \
  --condicion-iva monotributo
# agregá --production para el entorno real (por defecto: homologación/testing)
```

### 3. Generar clave privada + CSR

```bash
arquita generate-certificate
```

Genera **localmente** dos archivos en `~/arquita/certs/`:

- `arquita.key` — tu **clave privada**. No se comparte ni se sube a ningún lado; queda solo en tu máquina.
- `arquita.csr` — el **pedido de certificado** (PKCS#10) que vas a cargar en el portal de ARCA en el paso siguiente.

### 4. Habilitar el certificado en el portal de ARCA

#### Homologación (testing) — vía WSASS

El servicio de testing se gestiona con **WSASS** y **no es delegable**: hay que entrar con la **clave fiscal de la persona física** (no la de una empresa).

1. En el portal de ARCA, buscá y adherí el servicio **«WSASS – Autogestión Certificados Homologación»**.
2. Entrá a **«Nuevo Certificado»**: ingresá un **nombre simbólico de DN** y **pegá el contenido de `arquita.csr`** en el campo de solicitud PKCS#10. Descargá el certificado emitido a `~/arquita/certs/arquita.crt`. (ver [cómo generar el certificado](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/habilitar-administrador-de-certificados-de-testing))
3. Entrá a **«Crear autorización a servicio»**: seleccioná ese DN, ingresá el **CUIT representado** y elegí el servicio **`wsfe`** (ver [cómo autorizar el web service de testing](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/autorizar-web-service-de-testing)).

#### Producción — vía Administración de Certificados Digitales

1. Adherí y entrá a **«Administración de Certificados Digitales»** (ver [cómo habilitarlo](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/habilitar-administrador-de-certificados-de-produccion)). Creá un **alias**, subí `arquita.csr` y descargá el certificado emitido a `~/arquita/certs/arquita.crt` (ver [cómo generar el certificado](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/obtener-certificado-de-produccion#paso-4-generar-el-certificado-cert)).
2. Entrá a **«Administrador de Relaciones de Clave Fiscal»** y creá una relación que asocie ese certificado (como *computador fiscal*) al servicio **«wsfe – Factura Electrónica»** (ver [cómo autorizar el web service](https://docs.afipsdk.com/recursos/tutoriales-pagina-de-arca/autorizar-web-service-de-produccion)).

## Uso

```bash
# Estado del servicio (no requiere certificado)
arquita status

# Último comprobante autorizado
arquita last-voucher

# Emitir una Factura C a consumidor final por $1000
arquita create-voucher --importe 1000

# Factura C por servicios (período facturado + vencimiento de pago)
arquita create-voucher --importe 1000 --concepto servicios \
  --fecha-servicio-desde 20260701 \
  --fecha-servicio-hasta 20260731 \
  --fecha-vto-pago 20260810
```

De todos los datos del comprobante, lo único **obligatorio** es `--importe`; el
resto tiene default. Fechas en formato `YYYYMMDD`.

| Flag | Default | Descripción |
|------|---------|-------------|
| `--importe` | *(obligatorio)* | Importe total en pesos (para Factura C, neto = total). |
| `--concepto` | `productos` | `productos`, `servicios` o `productos-y-servicios`. |
| `--doc-tipo` | `consumidor-final` | Tipo de documento del receptor: `cuit`, `cuil`, `dni`, `consumidor-final`. |
| `--doc-nro` | `0` | Número de documento del receptor. |
| `--cond-iva-receptor` | `5` | Condición IVA del receptor (5 = consumidor final). |
| `--fecha` | hoy (AR) | Fecha del comprobante (`CbteFch`). |
| `--fecha-servicio-desde` | `--fecha` | Inicio del período (solo `servicios`). |
| `--fecha-servicio-hasta` | `--fecha` | Fin del período (solo `servicios`). |
| `--fecha-vto-pago` | `--fecha` | Vencimiento de pago (solo `servicios`). |

El **número de comprobante** se calcula solo.

## Licencia

MIT.
