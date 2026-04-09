# Pasos de Publicación - Scrivano Snap Store

Sigue estos pasos EXACTOS para publicar Scrivano en la Ubuntu Snap Store.

## Paso 1: Autenticación

```bash
snapcraft login
```

Se abrirá tu navegador para autenticación con Ubuntu One. Completa el login.

## Paso 2: Registrar el Nombre

**Solo ejecutar si es la primera vez que publicas:**

```bash
snapcraft register scrivano
```

Esto reserva el nombre "scrivano" en la tienda.

## Paso 3: Subir el Snap

```bash
snapcraft upload --release=stable snap/scrivano_1.1.8_amd64.snap
```

Este comando:
- ✅ Sube el snap a la tienda
- ✅ Lo publica en el canal stable
- ✅ Lo hace disponible públicamente

## Paso 4: Configurar en el Dashboard

### 4.1 Agregar Screenshots

1. Ve a: https://dashboard.snapcraft.io/snaps/scrivano/
2. Click en la pestaña **"Listing"**
3. Sube las screenshots:
   - Click en **"Add screenshot"**
   - Selecciona `screenshots/1.png`
   - Repite para las 4 screenshots
4. Click en **"Save changes"**

### 4.2 Configurar Categorías (Opcional)

En el mismo dashboard:
1. Click en **"Categories"**
2. Selecciona:
   - **Audio**
   - **Productivity**
3. Click en **"Save"**

### 4.3 Agregar Imagen de Banner (Opcional)

1. Click en **"Listing"**
2. Sube una imagen de banner (background)
3. Click en **"Save changes"**

## Paso 5: Verificar Publicación

```bash
# Ver información pública
snap info scrivano

# Instalar desde la tienda
sudo snap install scrivano

# Verificar que funciona
scrivano
```

Si todo funciona correctamente, ¡felicidades! Scrivano está publicado en la Snap Store.

## Comandos de Referencia Rápida

```bash
# Ver estado de canales
snapcraft status scrivano

# Ver revisiones
snapcraft list-revisions scrivano

# Promover entre canales
snapcraft release scrivano <revision> stable

# Ver logs del snap
journalctl -u snap.scrivano
```

## Troubleshooting

### Error: "snapcraft login credentials expired"

```bash
snapcraft logout
snapcraft login
```

### Error: "name 'scrivano' is already registered"

No es un error - ya está registrado. Continúa con el paso 3.

### Error: "upload failed"

Verifica que el snap existe:
```bash
ls -lh snap/scrivano_1.1.8_amd64.snap
```

El archivo debe existir y pesar ~7.5 MB.

## Checklist Final

- [ ] `snapcraft login` ejecutado
- [ ] `snapcraft register scrivano` ejecutado (solo primera vez)
- [ ] `snapcraft upload --release=stable snap/scrivano_1.1.8_amd64.snap` ejecutado
- [ ] Screenshots subidas en dashboard.snapcraft.io
- [ ] `snap info scrivano` muestra la información correcta
- [ ] `sudo snap install scrivano` funciona

---

**Documentación completa**: Ver `docs/snap-publishing-guide.md`