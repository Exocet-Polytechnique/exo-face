<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="battery-card">
        <BatteryComponent :data="batteryData" :modules-in-danger="modulesInDanger"/>
      </div>
      <div id="telemetry-card" class="grid-item">
        <TelemetryComponent :data="telemetryData"/>
      </div>
      <div id="position-card" class="grid-item">
        <PositionComponent :data="positionData"/>
      </div>
    </div>

  </div>
</template>

<script>

import BatteryComponent from './components/Battery.vue';
import PositionComponent from './components/Position.vue';
import TelemetryComponent from './components/Telemetry.vue';

export default {
  name: 'App',
  components: {
    BatteryComponent,
    PositionComponent,
    TelemetryComponent
  },
  data(){
    return{
      batteryData: [
        { id: 1, status: "Active", duration: "2h", voltage: "3.70", current: "1.23", temperature: "25" },
        { id: 2, status: "Inactive", duration: "N/A", voltage: "0.00", current: "0.00", temperature: "22" },
        { id: 3, status: "Active", duration: "2h", voltage: "3.70", current: "1.23", temperature: "25" },
        { id: 4, status: "Inactive", duration: "N/A", voltage: "0.00", current: "0.00", temperature: "22" },
        { id: 5, status: "Active", duration: "2h", voltage: "3.70", current: "1.23", temperature: "25" },
        { id: 6, status: "Inactive", duration: "N/A", voltage: "0.00", current: "0.00", temperature: "22" },
      ],
      modulesInDanger: [2, 4, 6],
      positionData:{latitude: 45.502991, longitude: -73.613991},
      telemetryData:{speed: 80.0, h2: 60}
    }
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
