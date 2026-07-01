<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="battery-card">
        <BatteryComponent :data="batteryData"/>
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
      this.batteryData = object.modules.filter((module) => module.id !== 0).map((module) => ({
        let name = "";
        let max = 65;

        if (module.id === 1 || module.id === "1") {
          name = "Télémétrie";
          max = 65;
        } else if (module.id === 2 || module.id === "2") {
          name = "Auxiliaire";
          max = 60;
        } else if (module.id === 3 || module.id === "3") {
          name = "Stockage";
          max = 70;
        }

        return {
          id: module.id,
          displayName: name,
          status: module.status,
          duration: module.estimated_life ? `${module.estimated_life}h` : "N/A",
          voltage: module.voltage.toFixed(2),
          current: module.current.toFixed(2),
          temperature: module.temperature.toFixed(0),
          maxTemp: max
        };
      }));

      this.modulesInDanger = object.modules
        .filter((module) => module.status !== "Active")
        .map((module) => module.id);
      //TODO: Use real calculations to determine whether the data is valid

      this.positionData = {
        latitude: object.latitude,
        longitude: object.longitude,
      };

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
