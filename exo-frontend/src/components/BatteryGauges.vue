<template>
    <div id="container">
        <h1 id="title">Batteries</h1>
        <div id="gauges">
            <div class="gauge-block">
                <VueSpeedometer
                dimensionUnit="px"
                :width="250"
                :height="150"
                :maxSegmentLabels="5"
                :segments="5"
                :needleHeightRatio="0.5"
                :value="data.telemetryBatteryCharge"
                :maxValue="100"
                :segmentColors='["#e51f1f", "#f7e379", "#44ce1b", "#44ce1b", "#44ce1b"]'
                :paddingVertical="15"
                valueFormat=".1f"
                currentValueText="Batterie Télémétrie: ${value}%"
                valueTextFontSize="3vh"
                valueTextFontWeight="500"
                ></VueSpeedometer>
                <div class="details">
                    <span>Tension: {{ data.telemetryBatteryVoltage?.toFixed(2) ?? '--' }} V</span>
                    <span>Courant: {{ data.telemetryBatteryCurrent?.toFixed(2) ?? '--' }} A</span>
                    <span>Puissance: {{ data.telemetryBatteryPower?.toFixed(1) ?? '--' }} W</span>
                    <span>Température: {{ data.telemetryBatteryTemperature?.toFixed(1) ?? '--' }} °C</span>
                </div>
            </div>
            <div class="gauge-block">
                <VueSpeedometer
                dimensionUnit="px"
                :width="250"
                :height="150"
                :maxSegmentLabels="5"
                :segments="5"
                :needleHeightRatio="0.5"
                :value="data.auxBatteryCharge"
                :maxValue="100"
                :segmentColors='["#e51f1f", "#f7e379", "#44ce1b", "#44ce1b", "#44ce1b"]'
                :paddingVertical="15"
                valueFormat=".1f"
                currentValueText="Batterie Auxiliaire: ${value}%"
                valueTextFontSize="3vh"
                valueTextFontWeight="500"
                ></VueSpeedometer>
                <div class="details">
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

<style>

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
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
    align-items: flex-start;
}

.gauge-block {
    display: flex;
    flex-direction: column;
    align-items: center;
}

.details {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: #313239;
    font-size: 2.2vh;
}

</style>
