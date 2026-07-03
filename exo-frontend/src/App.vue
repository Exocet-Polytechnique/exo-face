<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="battery-card">
        <BatteryComponent :data="batteryData" :nowMs="nowMs" />
      </div>
      <div id="telemetry-card" class="grid-item">
        <TelemetryComponent :data="telemetryData"/>
      </div>
      <div id="position-card" class="grid-item">
        <PositionComponent/>
      </div>
    </div>

  </div>
</template>

<script>

import BatteryComponent from './components/Battery.vue';
import PositionComponent from './components/Position.vue';
import TelemetryComponent from './components/Telemetry.vue';
import { setupLogCapture } from './logCapture';

export default {
  name: 'App',
  components: {
    BatteryComponent,
    PositionComponent,
    TelemetryComponent
  },
  data(){
    return{
      batteryData: [],
      positionData:{latitude: null, longitude: null},
      telemetryData:{speed: null, h2: null},
      nowMs: 0
    }
  },
  created() {},
  mounted(){

    // Connect to the WebSocket
    this.socket = new WebSocket('ws://127.0.0.1:8000/');

    // Setup log capture to send console logs to server
    setupLogCapture(this.socket);

    this.socket.onmessage = (event) => {

      const object = JSON.parse(event.data);
      const now_ms = object.now_ms || Date.now();
      const boat = object.boat || object;
      // Mapping the received data
      this.batteryData = boat.modules
        .filter((module) => module.id !== 0)
        .map((module) => ({
          id: module.id,
          voltage: module.voltage,
          current: module.current,
          temperature: module.temperature.toFixed(1),
          last_error_ms: module.last_error_ms || 0
        }));

      this.telemetryData = {
        speed: boat.speed,
        h2: boat.hydrogen_level,
      };

      this.nowMs = now_ms;
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
  grid-template-columns: 1fr 1fr 2fr;
  grid-template-rows: 1fr 2fr;
  column-gap: 1vh;
  row-gap: 1vh;
  height: 98vh;
}

#battery-card {
  background-color: white;
  grid-column: 1 / 4;
  grid-row: 2 / 3; 
  border-radius: 10px;
}

#telemetry-card {
  background-color: white;
  grid-column: 2 / 4;
  grid-row: 1 / 2;
  border-radius: 10px;
}

#position-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 1 / 2;
  border-radius: 10px;
}

</style>
