<template>
    <div id="container">
        <h1 id="title">Batteries</h1>
        <table id="battery-table" cla>
            <thead>
                <tr id="battery-table-header">
                    <th>Module</th>
                    <th>Statut</th>
                    <th>Dur. Est.</th>
                    <th>Voltage(V)</th>
                    <th>Courant(A)</th>
                    <th>Temp.(°C)</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="row in data" :key="row.id" :class="{ danger: modulesInDanger.includes(row.id)}">
                    <td>{{ row.id }}</td>
                    <td>{{ row.status }}</td>
                    <td>{{ row.duration }}</td>
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
    modulesInDanger:{
        type: Array,
        required: false
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
</style>