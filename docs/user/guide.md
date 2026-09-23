# Guía de Usuario de NetworkBench

NetworkBench permite diagnosticar con precisión profesional la velocidad real y la calidad de la conexión de red entre dos equipos con Windows.

---

## 1. Primeros Pasos

### 1.1 Iniciar la Aplicación en Ambos Equipos
Para realizar una medición de red, debe ejecutar NetworkBench en dos equipos conectados a la misma red local (LAN, Wi-Fi o VPN):
- Un equipo actuará como **Iniciador** (quien solicita la prueba).
- El otro equipo actuará como **Receptor** (quien responde y confirma la prueba).

Ambos equipos ejecutan el mismo programa y pueden intercambiar roles en cualquier momento.

### 1.2 Descubrimiento y Emparejamiento
1. En la pestaña **Equipos**, NetworkBench busca automáticamente otros equipos cercanos mediante mDNS. Si su equipo aparece en la lista, pulse **Conectar**.
2. Si los equipos están en subredes distintas o el descubrimiento automático está deshabilitado, introduzca directamente la dirección IPv4, IPv6 o nombre DNS del equipo remoto y pulse **Conectar**.
3. Ambos equipos mostrarán un **código de seguridad de 6 dígitos**. Verifique visualmente o por teléfono/mensajería que el código coincide en ambas pantallas y pulse **Aceptar**.
4. La conexión queda autenticada e inmune a ataques de intermediario (*Man-in-the-Middle*).

---

## 2. Ejecución de Pruebas

### 2.1 Modo Estándar (Recomendado)
- Seleccione el plan estándar: mide la velocidad TCP en ambas direcciones secuencialmente (Ida y Vuelta).
- La prueba dura 10 segundos por dirección con un periodo de estabilización inicial para descartar efectos de ráfaga de inicio lento (TCP Slow Start).

### 2.2 Modo Avanzado
Para usuarios técnicos y administradores de red:
- **Protocolo**: Elija entre **TCP** o **UDP**.
- **Streams Concurrentes**: Configure de 1 a 64 hilos paralelos (1 a 32 en modo simultáneo).
- **Dirección**:
  - *Secuencial*: Primero A hacia B, luego B hacia A.
  - *Simultáneo (`RunningBoth`)*: Tráfico cruzado bidireccional en tiempo real para detectar problemas de full-duplex y saturación de buffers (Bufferbloat).
- **Pruebas UDP**: Configure el tamaño de paquete (datagrama) y la tasa objetivo. NetworkBench reportará paquetes enviados, recibidos y porcentaje exacto de pérdida.

---

## 3. Interpretación de Veredictos y Métricas

Al finalizar la prueba, NetworkBench muestra un informe visual claro estructurado en dos niveles:

### Nivel 1: Veredicto Global
- 🟢 **Excelente**: La red rinde al máximo esperado de su capacidad de enlace (Gigabit, 2.5 Gbps, etc.) con estabilidad perfecta y sin pérdidas.
- 🟡 **Bueno con Observaciones**: La red funciona pero se detectaron pequeñas anomalías (por ejemplo, asimetría leve entre subida y bajada, o una sobrecarga ligera de CPU).
- 🔴 **Problemas Detectados**: Existen problemas que degradan significativamente la conexión (pérdida apreciable de paquetes en UDP, colapso de caudal o retransmisiones severas).

### Nivel 2: Diagnóstico Detallado
El panel muestra tres bloques explicativos:
1. **Hechos**: Mediciones numéricas objetivas y verificadas (tasa media, pérdidas, variabilidad).
2. **Posibles Causas**: Hipótesis técnicas probables (cable Ethernet degradado negociando a 100 Mbps, saturación del conmutador, interferencias Wi-Fi o antivirus inspeccionando paquetes).
3. **Acciones Sugeridas**: Pasos concretos para aislar o reparar el fallo.

---

## 4. Historial y Exportación de Informes

- **Historial**: Consulte todas las mediciones previas agrupadas por fecha. Compare automáticamente mediciones entre los mismos dos equipos para ver si la red mejoró o empeoró con el tiempo.
- **Exportación**:
  - **PDF**: Documento maquetado profesional listo para imprimir o enviar por correo electrónico.
  - **JSON**: Datos estructurados para análisis automatizado o archivado.
  - **CSV**: Compatible con Microsoft Excel en español e inglés, protegido contra inyección de fórmulas maliciosas.
  - **Opción de Anonimización**: Permite eliminar nombres de host, direcciones IP y huellas digitales antes de compartir el informe externamente.
