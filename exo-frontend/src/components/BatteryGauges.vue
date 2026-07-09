<template>
    <div id="container">
        <h1 id="title">Batteries</h1>
        <div id="gauges">
            <div class="gauge-block">
                <VueSpeedometer
                dimensionUnit="px"
                :width="220"
                :height="132"
                :maxSegmentLabels="5"
                :segments="5"
                :needleHeightRatio="0.5"
                :value="data.telemetryBatteryCharge"
                :maxValue="100"
                :segmentColors='["#e51f1f", "#f7e379", "#44ce1b", "#44ce1b", "#44ce1b"]'
                :paddingVertical="15"
                valueFormat=".1f"
                currentValueText=""
                ></VueSpeedometer>
                <div class="details">
                    <span>Batterie Télémétrie</span>
                    <span>Charge: {{ data.telemetryBatteryCharge?.toFixed(1) ?? '--' }} %</span>
                    <span>Puissance: {{ data.telemetryBatteryPower?.toFixed(1) ?? '--' }} W</span>
                    <span>Température: {{ data.telemetryBatteryTemperature?.toFixed(1) ?? '--' }} °C</span>
                </div>
            </div>
            <div class="gauge-block">
                <VueSpeedometer
                dimensionUnit="px"
                :width="220"
                :height="132"
                :maxSegmentLabels="5"
                :segments="5"
                :needleHeightRatio="0.5"
                :value="data.auxBatteryCharge"
                :maxValue="100"
                :segmentColors='["#e51f1f", "#f7e379", "#44ce1b", "#44ce1b", "#44ce1b"]'
                :paddingVertical="15"
                valueFormat=".1f"
                currentValueText=""
                ></VueSpeedometer>
                <div class="details">
                    <span>Batterie Auxiliaire</span>
                    <span>Charge: {{ data.auxBatteryCharge?.toFixed(1) ?? '--' }} %</span>
                    <span>Puissance: {{ data.auxBatteryPower?.toFixed(1) ?? '--' }} W</span>
                    <span>Température: {{ data.auxBatteryTemperature?.toFixed(1) ?? '--' }} °C</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script>

import VueSpeedometer from "vue-speedometer"

export default {
  name: 'BatteryGaugesComponent',
  props: {
    data: {
        type: Object,
        required: true
    }
  },
  components: {
    VueSpeedometer
  }
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
