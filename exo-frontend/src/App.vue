<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="battery-card">
        <BatteryComponent :data="batteryData" :modules-in-danger="modulesInDanger"/>
      </div>
    </div>

  </div>
</template>

<script>

import BatteryComponent from './components/Battery.vue';

export default {
  name: 'App',
  components: {
    BatteryComponent,
  },
  data(){
    return{
      batteryData: [],
      modulesInDanger: [],
    }
  },
  mounted(){

    // Connect to the WebSocket
    this.socket = new WebSocket('ws://127.0.0.1:8000/');

    this.socket.onmessage = (event) => {

      const object = JSON.parse(event.data);
      // Mapping the received data
      this.batteryData = object.modules.map((module) => ({
        id: module.id,
        status: module.status,
        duration: module.estimated_life ? `${module.estimated_life}h` : "N/A",
        voltage: module.voltage.toFixed(2),
        current: module.current.toFixed(2),
        temperature: module.temperature.toFixed(0),
      }));

      this.modulesInDanger = object.modules
        .filter((module) => module.status !== "Active")
        .map((module) => module.id);
      //TODO: Use real calculations to determine whether the data is valid
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
  column-gap: 1vh;
  row-gap: 1vh;
  height: 98vh;
}

#battery-card {
  background-color: white;
  border-radius: 10px;
}

</style>
