<template>
  <div id="app">
    <div id="temperature-display">
      <span 
        id="temperature-value"
        :class="{ 'temperature-alert': batteryGauges.auxBatteryTemperature > 54 }"
      >
        {{ batteryGauges.auxBatteryTemperature?.toFixed(1) ?? '--' }} °C
      </span>
    </div>
  </div>
</template>

<script>

import { setupLogCapture } from './logCapture';

export default {
  name: 'App',
  components: {
  },
  data(){
    return{
      pcbStatus: [],
      boatState: 0,
      positionData:{latitude: null, longitude: null},
      batteryGauges:{
        auxBatteryTemperature: 55,
      }
    }
  },
  mounted(){

    // Connect to the WebSocket
    this.socket = new WebSocket('ws://127.0.0.1:8000/');

    // Setup log capture to send console logs to server
    setupLogCapture(this.socket);

    this.socket.onmessage = (event) => {

      const object = JSON.parse(event.data);
      // Mapping the received data
      this.pcbStatus = object.pcb_status;
      this.boatState = object.boat_state;

      this.batteryGauges = {
        auxBatteryCharge: object.aux_battery_charge,
        auxBatteryTemperature: object.aux_battery_temperature,
        auxBatteryPower: object.aux_battery_power,
        telemetryBatteryCharge: object.telemetry_battery_charge,
        telemetryBatteryVoltage: object.telemetry_battery_voltage,
        telemetryBatteryCurrent: object.telemetry_battery_current,
        telemetryBatteryPower: object.telemetry_battery_power,
        telemetryBatteryTemperature: object.telemetry_battery_temperature,
      };
    }

    this.socket.onopen = () => {
      this.socket.send('Ping');
    }

    this.socket.onerror = () => {
      console.error('An error occurred while connecting to the WebSocket server.');
    }

    this.socket.onclose = () => {
      console.log('Disconnected from the WebSocket server.');
    }

    // For initial logging setup
    console.log('Dashboard initialized and connected to server');
  }
}
</script>

<style>
html, body {
  overflow: hidden;
  margin: 0;
  padding: 0;
}

body {
  background-color: white;
}

#app {
  width: 100vw;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: white;
}

#temperature-display {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

#temperature-value {
  font-size: 35vh;
  font-weight: bold;
  color: black;
  font-family: 'Courier New', monospace;
  white-space: nowrap;
}

#temperature-unit {
  font-size: 15vh;
  color: black;
  font-family: 'Courier New', monospace;
}

.temperature-alert {
  color: red !important;
  animation: blink 0.5s infinite;
}

@keyframes blink {
  0%, 49% {
    opacity: 1;
  }
  50%, 100% {
    opacity: 0;
  }
}
</style>
