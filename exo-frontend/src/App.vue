<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="telemetry-card" class="grid-item">
        <BatteryGaugesComponent :data="batteryGauges"/>
      </div>
      <div id="boat-state-card">
        <BoatStateComponent :state="boatState"/>
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
import BoatStateComponent from './components/BoatState.vue';
import { setupLogCapture } from './logCapture';

export default {
  name: 'App',
  components: {
    AlertsComponent,
    BatteryGaugesComponent,
    BoatStateComponent
  },
  data(){
    return{
      pcbStatus: [],
      boatState: 0,
      positionData:{latitude: null, longitude: null},
      batteryGauges:{
        auxBatteryCharge: 0,
        auxBatteryTemperature: 0,
        auxBatteryPower: 0,
        telemetryBatteryCharge: 0,
        telemetryBatteryVoltage: 0,
        telemetryBatteryCurrent: 0,
        telemetryBatteryPower: 0,
        telemetryBatteryTemperature: 0,
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
}

body {
  background-color: #313239;
  margin: 1vh;
}

#grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: minmax(0, 3fr) minmax(0, 1fr) minmax(0, 3fr);
  column-gap: 1vh;
  row-gap: 1vh;
  height: 98vh;
}

#telemetry-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 1 / 2;
  border-radius: 10px;
  overflow: hidden;
}

#boat-state-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 2 / 3;
  border-radius: 10px;
  overflow: hidden;
}

#battery-card {
  background-color: white;
  grid-column: 1 / 2;
  grid-row: 3 / 4;
  border-radius: 10px;
  overflow: hidden;
}

</style>
