<template>
    <div id="container">
        <h1 id="title">Batteries</h1>
        <table id="battery-table">
            <thead>
                <tr id="battery-table-header">
                    <th>Module</th>
                    <th>Voltage(V)</th>
                    <th>Courant(A)</th>
                    <th>Temp.(°C)</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="row in data" :key="row.id ">
                    <td :class="{'blink': isBlinking(row)}">{{ row.id }}</td>
                    <td>{{ row.voltage }}</td>
                    <td>{{ row.current }}</td>
                    <td>{{ row.temperature }}</td>
                  </tr>
            </tbody>
        </table>
    </div>
</template>

<script>
export default {
  name: 'BatteryComponent',
  props: {
    data: {
      type: Array,
      required: true
    },
    nowMs: {
      type: Number,
      required: false,
      default: 0
    }
  },
  methods: {
    isBlinking(row) {
      if (!row || !row.last_error_ms || !this.nowMs) return false;
      const delta = this.nowMs - row.last_error_ms;
      return delta >= 0 && delta < 5000; // blink for 5s
    }
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

#battery-table {
  width: 100%;
  border-collapse: collapse;
}

#battery-table th {
  padding-top: 0.3vh;
  padding-left: 0.3vh;
  padding-bottom: 0.3vh;
  text-align: left;
  color: white;
  font-size: 5vh;
}

#battery-table-header {
    background-color: #c12736;
}
.danger {
    background-color: #fe7e44;
}

#battery-table td {
  padding-top: 0.3vh;
  padding-left: 0.3vh;
  padding-bottom: 0.3vh;
  text-align: left;
  font-size: 5vh;
}

.blink {
  animation: blink-bg 1s steps(2, start) 0s infinite;
}

@keyframes blink-bg {
  50% { background-color: #fe7e44; }
}
</style>