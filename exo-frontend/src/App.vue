<template>
  <div id="app">
    <div id="grid-container" class="grid-item">
      <div id="battery-card">
        <h1>Battery</h1>
      </div>
      <div id="telemetry-card" class="grid-item">
        <h1>Telemetry</h1>
      </div>
      <div id="position-card" class="grid-item">
        <h1>Position</h1>
      </div>
    </div>

  </div>
</template>

<script>

export default {
  name: 'App',
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
