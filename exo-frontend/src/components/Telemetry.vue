<template>
    <div id="container">
        <h1 id="title">Télémétrie</h1>
        <div id="telemetry">
            <VueSpeedometer
            :dimensionUnit="px"
            :width="250"
            :height="150"
            :maxSegmentLabels="10"
            :segments="10"
            :needleHeightRatio="0.8"
            :needleColor="'#c02737'"
            :value="data.speed"
            :maxValue="200"
            :segmentColors='["#303238"]'
            :paddingVertical="15"
            valueTextFontSize="10vh"
            valueTextFontWeight="500"
            ></VueSpeedometer>
            <VueSpeedometer
            :dimensionUnit="px"
            :width="250"
            :height="150"
            :maxSegmentLabels="5"
            :segments="5"
            :needleHeightRatio="0.5"
            :value="data.h2"
            :maxValue="100"
            :segmentColors='["#e51f1f", "#f7e379", "#44ce1b", "#44ce1b", "#44ce1b"]'
            :paddingVertical="15"
            valueTextFontSize="10vh"
            valueTextFontWeight="500"
            ></VueSpeedometer>
            
        </div>
    </div>
</template>

<script>

import VueSpeedometer from "vue-speedometer"

export default {
  name: 'TelemetryComponent',
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

#telemetry {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
    align-items: center;
}

</style>