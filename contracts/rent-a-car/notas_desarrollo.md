# Notas de Desarrollo

Este archivo documenta cambios y configuraciones realizadas fuera del contenido del curso.

## Configuración de TypeScript

### Fecha: 2 de noviembre, 2025

**Cambio realizado:** Deshabilitado `useUnknownInCatchVariables` en `tsconfig.app.json`

**Archivo modificado:** `tsconfig.app.json`

**Razón:** El curso utiliza una configuración de TypeScript donde los errores en bloques `catch` son de tipo `any` por defecto. Nuestro proyecto tenía habilitada la opción `useUnknownInCatchVariables` (incluida en `strict: true`), lo que causaba errores de tipo al intentar acceder a propiedades del error sin type guards.

**Problema original:**

```typescript
catch (error) {
  // Error: 'error' is of type 'unknown'
  if (error.response?.data?.extras?.result_codes) {
    // ...
  }
}
```

**Solución:** Se agregó la línea en `tsconfig.app.json`:

```json
"useUnknownInCatchVariables": false
```

Esto permite que el código del curso funcione sin modificaciones, manteniendo los errores como tipo `any` en lugar de `unknown`.

**Archivos afectados:**

- `src/services/stellar.service.ts` - Método `submitTransaction` (línea 66)
- Otros métodos con bloques catch en el mismo archivo

---

## Checkpoint: Versión Base Funcional

### Fecha: 2 de noviembre, 2025 - 1:24 AM

**Estado:** Versión base del contrato rent-a-car completamente funcional.

**Funcionalidades implementadas:**

- ✅ Inicialización del contrato (`__constructor`)
- ✅ Agregar autos (`add_car`)
- ✅ Remover autos (`remove_car`)
- ✅ Consultar estado de auto (`get_car_status`)
- ✅ Alquilar auto (`rental`)
- ✅ Retiro de fondos del owner (`payout_owner`)
- ✅ Frontend funcional con roles: Admin, Owner, Renter

**Próximas mejoras a implementar:**

1. Sistema de comisiones del administrador
2. Cobro automático de comisión en cada alquiler
3. Función de retiro de comisiones para el admin
4. Validación de retiro de owners (solo cuando auto está devuelto)
5. Función para devolver auto

**Commit:** Versión base funcional antes de implementar sistema de comisiones

---
