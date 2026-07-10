<template>
    <div id="container">
        <h1 id="title">Batteries</h1>
        <div id="gauges">
            <span> {{ data.auxBatteryTemperature?.toFixed(1) ?? '--' }} °C</span>
        </div>
    </div>
</template>

<script>


export default {
  name: 'BatteryGaugesComponent',
  props: {
    data: {
        type: Object,
        required: true
    }
  },
}
</script>

<style scoped>

#container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

#title {
    margin: 0.5vh;
    padding: 0;
    text-align: left;
    font-size: 6vh;
    color: #313239;
}

#gauges {
    width: 100%;
    flex: 1;
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
    align-items: center;
    font-size: 15vh;
}

.gauge-block {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 3vh;
}

.details {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    color: #313239;
    font-size: 4vh;
}

.details span:first-child {
    font-weight: 700;
    margin-bottom: 0.5vh;
}

</style>
