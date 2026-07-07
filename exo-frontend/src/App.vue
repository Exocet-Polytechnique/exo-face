<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="telemetry-card" class="grid-item">
        <TelemetryComponent :data="telemetryData"/>
      </div>
      <div id="battery-card">
        <BatteryComponent :data="batteryData"/>
      </div>
    </div>

  </div>
</template>

<script>

import BatteryComponent from './components/Battery.vue';
import TelemetryComponent from './components/Telemetry.vue';
import { setupLogCapture } from './logCapture';

export default {
  name: 'App',
  components: {
    BatteryComponent,
    TelemetryComponent
  },
  data(){
    return{
      batteryData: [],
      positionData:{latitude: null, longitude: null},
      telemetryData:{speed: null, h2: null}
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
      this.batteryData = object.modules
        .filter((module) => module.id !== 0)
        .map((module) => ({
          id: module.id,
          voltage: module.voltage,
          current: module.current,
          temperature: module.temperature.toFixed(1),
        }));

      this.telemetryData = {
        speed: object.speed,
        h2: object.hydrogen_level,
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
body {
  background-color: #313239;
  margin: 1vh;
}

#grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
  column-gap: 1vh;
  row-gap: 1vh;
  height: 98vh;
}

#battery-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 2 / 3;
  border-radius: 10px;
}

#telemetry-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 1 / 2;
  border-radius: 10px;
}

</style>
