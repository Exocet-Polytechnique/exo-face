<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="telemetry-card" class="grid-item">
        <BatteryGaugesComponent :data="batteryGauges"/>
      </div>
      <div id="battery-card">
        <AlertsComponent :data="pcbStatus"/>
      </div>
    </div>

  </div>
</template>

<script>

import AlertsComponent from './components/Alerts.vue';
import BatteryGaugesComponent from './components/BatteryGauges.vue';
import { setupLogCapture } from './logCapture';

export default {
  name: 'App',
  components: {
    AlertsComponent,
    BatteryGaugesComponent
  },
  data(){
    return{
      pcbStatus: [],
      positionData:{latitude: null, longitude: null},
      batteryGauges:{auxBatteryCharge: 0, telemetryBatteryCharge: 0}
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

      this.batteryGauges = {
        auxBatteryCharge: object.aux_battery_charge,
        telemetryBatteryCharge: object.telemetry_battery_charge,
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
